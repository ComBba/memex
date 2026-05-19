//! Path sandbox for IPC-supplied filesystem paths (KF-01).
//!
//! Every Tauri command that resolves a filesystem path from frontend or
//! Qdrant payload routes the path through [`validate_session_path`]. The
//! validator canonicalizes the candidate (resolving symlinks, `..` and `.`
//! components, and any case/HFS+ folding the OS applies) and then confirms
//! the canonical form lives under the sandbox root.
//!
//! Sandbox root: `<home>/.claude/projects`, resolved via [`dirs::home_dir`]
//! so the same code path works on Linux, macOS, and Windows (where `$HOME`
//! is conventionally absent in favor of `%USERPROFILE%`).
//!
//! Threat model addressed:
//!   * A tampered Qdrant payload directing `get_session_turns` /
//!     `predict_next_actions` to read arbitrary files on disk.
//!   * A symlink planted inside the sandbox root that points outside —
//!     canonicalization resolves the link target first, then the
//!     starts-with check fails because the target escapes the root.
//!   * `..` traversal slipped through string concatenation upstream.
//!   * Embedded NUL bytes that Rust's `Path` accepts at the type level
//!     but that downstream POSIX syscalls would truncate.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

/// Canonicalized allow-list root for session-JSONL reads.
///
/// Construct via [`SandboxRoot::from_env`] in production, or
/// [`SandboxRoot::from_root`] in tests where the real `$HOME` should not
/// be touched.
#[derive(Debug, Clone)]
pub struct SandboxRoot {
    canonical_root: PathBuf,
}

impl SandboxRoot {
    /// Discover the sandbox root from the platform home directory.
    ///
    /// Returns `Err` if the home directory cannot be resolved or if
    /// `<home>/.claude/projects` does not exist. A nonexistent root is an
    /// error rather than silent acceptance — it would otherwise mean every
    /// path is "outside the sandbox" with a confusing error message.
    pub fn from_env() -> Result<Self> {
        let home = dirs::home_dir().context("could not resolve home directory")?;
        let claude_root = home.join(".claude").join("projects");
        let canonical_root = claude_root
            .canonicalize()
            .with_context(|| format!("canonicalize sandbox root {}", claude_root.display()))?;
        Ok(Self { canonical_root })
    }

    /// Construct from an explicit canonical path.
    ///
    /// Intended for integration tests that want to point the sandbox at a
    /// `tempfile::TempDir` without touching the real `$HOME`. The caller
    /// is responsible for canonicalizing the path beforehand — this
    /// constructor does not re-canonicalize so the test harness has full
    /// control.
    ///
    /// Not on the IPC happy path: production code must always go through
    /// [`SandboxRoot::from_env`].
    #[doc(hidden)]
    pub fn from_root(root: PathBuf) -> Self {
        Self {
            canonical_root: root,
        }
    }

    /// Returns the canonical path if `p` lives inside the sandbox root.
    ///
    /// Rejection reasons (each with a distinct error message):
    ///   * empty path
    ///   * path contains a NUL byte (Unix; on Windows `Path::new` would
    ///     have already panicked)
    ///   * `canonicalize` failed (typically: path does not exist)
    ///   * canonical form is not a descendant of the sandbox root
    ///     (catches symlink-escape and `..` traversal post-canonicalize)
    pub fn contains(&self, p: &Path) -> Result<PathBuf> {
        let s = p.as_os_str();
        if s.is_empty() {
            bail!("path is empty");
        }

        // NUL-byte rejection on the raw OS bytes. `Path::new("foo\0bar")`
        // is constructible on Unix and would pass downstream `open(2)` a
        // string that gets truncated at the NUL — a classic injection
        // vector. On Windows the constructor itself rejects NULs at the
        // type level, so the check is unnecessary there.
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            if s.as_bytes().contains(&0) {
                bail!("path contains NUL byte: {}", p.display());
            }
        }

        let canon = p
            .canonicalize()
            .with_context(|| format!("canonicalize {}", p.display()))?;

        // Critical: re-check containment *after* canonicalization. This is
        // what defeats symlink escape. A symlink inside the sandbox that
        // points to `/etc/passwd` canonicalizes to `/etc/passwd`, which
        // fails `starts_with(sandbox_root)`.
        if canon.starts_with(&self.canonical_root) {
            Ok(canon)
        } else {
            Err(anyhow!("path outside sandbox: {}", canon.display()))
        }
    }

    /// Public read of the configured root (canonicalized).
    pub fn root(&self) -> &Path {
        &self.canonical_root
    }
}

/// Convenience wrapper used by every IPC entry point. Reads
/// [`SandboxRoot::from_env`] fresh on each call (cheap: one `stat` on a
/// directory). Returns the canonicalized path so callers can pass the
/// canonical form to `parser::parse_session` and avoid a second
/// canonicalize round-trip.
///
/// SECURITY: always pass the returned canonical `PathBuf` to the
/// downstream parser, never the original `p`. Using the original allows
/// a TOCTOU racer to swap the path between validation and read; using
/// the canonical form locks in the resolved target.
pub fn validate_session_path(p: &Path) -> Result<PathBuf> {
    SandboxRoot::from_env()?.contains(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a tempdir-backed SandboxRoot so the tests don't depend on the
    /// real `$HOME` contents (and so CI without `~/.claude/projects` runs
    /// the same code path as a developer machine).
    fn make_sandbox() -> (TempDir, SandboxRoot) {
        let td = TempDir::new().expect("tempdir");
        let claude_root = td.path().join("claude_projects");
        fs::create_dir_all(&claude_root).unwrap();
        let canonical = claude_root.canonicalize().unwrap();
        (td, SandboxRoot::from_root(canonical))
    }

    #[test]
    fn t_valid_claude_session_path() {
        let (td, sb) = make_sandbox();
        let p = td.path().join("claude_projects/abc/sess.jsonl");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "{}\n").unwrap();
        let canon = sb.contains(&p).expect("inside sandbox");
        assert!(canon.starts_with(sb.root()));
    }

    #[test]
    fn t_path_outside_sandbox_etc() {
        let (_td, sb) = make_sandbox();
        // /etc/passwd exists on macOS/Linux but is outside the sandbox.
        let p = PathBuf::from("/etc/passwd");
        let err = sb.contains(&p).expect_err("must reject /etc/passwd");
        assert!(
            format!("{err}").contains("outside sandbox"),
            "error was: {err}"
        );
    }

    #[test]
    fn t_path_outside_both_tmp() {
        let (_td, sb) = make_sandbox();
        // A real file in a *different* tempdir is definitionally outside
        // the sandbox.
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("foo.jsonl");
        fs::write(&p, "x").unwrap();
        assert!(sb.contains(&p).is_err());
    }

    #[test]
    fn t_path_traversal_dotdot() {
        let (td, sb) = make_sandbox();
        // Plant a file inside the sandbox so the directory exists, then
        // construct a sibling file outside via `..` and demonstrate that
        // canonicalize resolves it to the outside location, which then
        // fails `starts_with(root)`.
        let inside = td.path().join("claude_projects/abc/sess.jsonl");
        fs::create_dir_all(inside.parent().unwrap()).unwrap();
        fs::write(&inside, "x").unwrap();
        fs::write(td.path().join("outside.jsonl"), "x").unwrap();
        let traversal = td.path().join("claude_projects/abc/../../outside.jsonl");
        assert!(sb.contains(&traversal).is_err());
    }

    #[test]
    fn t_path_traversal_double_dotdot() {
        let (td, sb) = make_sandbox();
        let inside = td.path().join("claude_projects/abc/sess.jsonl");
        fs::create_dir_all(inside.parent().unwrap()).unwrap();
        fs::write(&inside, "x").unwrap();
        // Deep traversal: keep going until we definitely land outside.
        let traversal =
            PathBuf::from("/etc/../etc/passwd");
        assert!(sb.contains(&traversal).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn t_symlink_outside() {
        let (td, sb) = make_sandbox();
        let outside_dir = TempDir::new().unwrap();
        let outside_file = outside_dir.path().join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();
        let link = td.path().join("claude_projects/link.jsonl");
        std::os::unix::fs::symlink(&outside_file, &link).unwrap();
        assert!(
            sb.contains(&link).is_err(),
            "symlink escaping sandbox must be rejected"
        );
    }

    #[test]
    #[cfg(unix)]
    fn t_symlink_inside() {
        let (td, sb) = make_sandbox();
        let real = td.path().join("claude_projects/def/foo.jsonl");
        fs::create_dir_all(real.parent().unwrap()).unwrap();
        fs::write(&real, "x").unwrap();
        let link = td.path().join("claude_projects/abc/link.jsonl");
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let canon = sb.contains(&link).expect("internal symlink accepted");
        // The canonical form resolves to the real file, which is inside.
        assert!(canon.starts_with(sb.root()));
    }

    #[test]
    #[cfg(unix)]
    fn t_symlink_dangling() {
        let (td, sb) = make_sandbox();
        let link = td.path().join("claude_projects/dangling.jsonl");
        std::os::unix::fs::symlink("/nonexistent/path/here", &link).unwrap();
        // canonicalize fails because the target doesn't exist.
        assert!(sb.contains(&link).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn t_nul_byte_path() {
        let (_td, sb) = make_sandbox();
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let p = PathBuf::from(OsString::from_vec(b"foo\0bar.jsonl".to_vec()));
        let err = sb.contains(&p).expect_err("NUL byte must be rejected");
        assert!(format!("{err}").contains("NUL"), "error was: {err}");
    }

    #[test]
    fn t_empty_string() {
        let (_td, sb) = make_sandbox();
        let p = PathBuf::from("");
        let err = sb.contains(&p).expect_err("empty path must be rejected");
        assert!(format!("{err}").contains("empty"), "error was: {err}");
    }

    #[test]
    fn t_nonexistent_path() {
        let (td, sb) = make_sandbox();
        // Path looks like it's inside the sandbox but the file doesn't
        // exist — canonicalize fails.
        let p = td.path().join("claude_projects/does_not_exist.jsonl");
        assert!(sb.contains(&p).is_err());
    }

    #[test]
    fn t_canonical_idempotent() {
        let (td, sb) = make_sandbox();
        let p = td.path().join("claude_projects/abc/sess.jsonl");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "x").unwrap();
        let c1 = sb.contains(&p).unwrap();
        let c2 = sb.contains(&c1).unwrap();
        assert_eq!(c1, c2);
    }

    #[test]
    fn t_unicode_path_valid() {
        let (td, sb) = make_sandbox();
        let p = td
            .path()
            .join("claude_projects/세션-한국어-😀.jsonl");
        fs::write(&p, "x").unwrap();
        assert!(sb.contains(&p).is_ok());
    }

    #[test]
    fn t_long_path_no_panic() {
        let (_td, sb) = make_sandbox();
        // 4096-character path — must not panic; Err is acceptable.
        let long: String = "a".repeat(4096);
        let p = PathBuf::from(long);
        let _ = sb.contains(&p);
    }

    #[test]
    #[cfg(unix)]
    fn t_arbitrary_bytes_no_panic() {
        let (_td, sb) = make_sandbox();
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        // Random byte sequences that aren't valid UTF-8 must not panic.
        for bytes in [
            b"\xff\xfe\xff".as_ref(),
            b"a\nb\rc".as_ref(),
            b"\x7f".as_ref(),
            b"".as_ref(),
        ] {
            let p = PathBuf::from(OsString::from_vec(bytes.to_vec()));
            let _ = sb.contains(&p); // must not panic
        }
    }

    #[test]
    fn t_root_accessor_returns_canonical() {
        let (_td, sb) = make_sandbox();
        // root() returns the canonicalized form set at construction time.
        assert!(sb.root().is_absolute());
    }

    #[test]
    fn t_path_with_spaces() {
        let (td, sb) = make_sandbox();
        let p = td.path().join("claude_projects/my project/sess.jsonl");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "x").unwrap();
        assert!(sb.contains(&p).is_ok());
    }

    #[test]
    fn t_rejects_directory_outside_sandbox() {
        let (_td, sb) = make_sandbox();
        // A directory (not a file) outside the sandbox — must still reject.
        let p = PathBuf::from("/tmp");
        assert!(sb.contains(&p).is_err());
    }

    #[test]
    fn t_accepts_directory_inside_sandbox() {
        let (td, sb) = make_sandbox();
        // contains() doesn't distinguish file vs dir — it just enforces
        // sandbox containment. Callers (parse_session) handle file-vs-dir.
        let p = td.path().join("claude_projects/abc");
        fs::create_dir_all(&p).unwrap();
        assert!(sb.contains(&p).is_ok());
    }

    #[test]
    fn t_error_message_does_not_leak_etc_contents() {
        let (_td, sb) = make_sandbox();
        let err = sb.contains(Path::new("/etc/passwd")).unwrap_err();
        let msg = format!("{err}");
        // The error message contains the path (a public fact) but not file
        // contents — sanity check that we're not accidentally embedding a
        // read of the file in the error.
        assert!(!msg.to_lowercase().contains("root:"));
        assert!(!msg.contains("/bin/bash"));
    }
}

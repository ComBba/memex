//! Integration tests for the IPC path sandbox (KF-01).
//!
//! Unit-level coverage of `SandboxRoot` lives next to the implementation in
//! `src/sec.rs`. This file exercises the *public* `validate_session_path`
//! wrapper through a synthesized sandbox so we can verify the
//! end-to-end happy path and the CI-portable rejection paths without
//! depending on the real `$HOME`.
//!
//! Tests that *do* depend on the real `$HOME` (i.e. that `~/.claude/projects`
//! exists) are marked `#[ignore]` so CI doesn't false-positive on them.
//! Run them with `cargo test -- --ignored` on a developer machine.

use std::fs;
use std::path::PathBuf;

use memex_lib::sec::{validate_session_path, SandboxRoot};
use tempfile::TempDir;

/// Helper: build a tempdir-backed sandbox without touching `$HOME`.
fn temp_sandbox() -> (TempDir, SandboxRoot) {
    let td = TempDir::new().expect("tempdir");
    let claude_root = td.path().join("claude_projects");
    fs::create_dir_all(&claude_root).unwrap();
    let canonical = claude_root.canonicalize().unwrap();
    (td, SandboxRoot::from_root(canonical))
}

#[test]
fn it_sandbox_accepts_valid_path() {
    let (td, sb) = temp_sandbox();
    let p = td.path().join("claude_projects/proj-x/sess.jsonl");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, b"{}\n").unwrap();
    let canon = sb.contains(&p).expect("valid path");
    assert!(canon.starts_with(sb.root()));
}

#[test]
fn it_sandbox_rejects_etc_passwd() {
    let (_td, sb) = temp_sandbox();
    // /etc/passwd is a real file on macOS/Linux but is definitionally
    // outside any tempdir-backed sandbox.
    let p = PathBuf::from("/etc/passwd");
    let err = sb.contains(&p).expect_err("/etc/passwd must be rejected");
    assert!(
        format!("{err}").contains("outside sandbox"),
        "error was: {err}"
    );
}

#[test]
fn it_sandbox_rejects_nonexistent_path() {
    let (td, sb) = temp_sandbox();
    let p = td.path().join("claude_projects/never_created.jsonl");
    // canonicalize fails on nonexistent paths — should surface as Err.
    assert!(sb.contains(&p).is_err());
}

#[test]
fn it_sandbox_rejects_empty_path() {
    let (_td, sb) = temp_sandbox();
    let err = sb
        .contains(&PathBuf::new())
        .expect_err("empty path must be rejected");
    assert!(format!("{err}").contains("empty"), "error was: {err}");
}

#[test]
fn it_sandbox_rejects_dotdot_traversal() {
    let (td, sb) = temp_sandbox();
    let inside = td.path().join("claude_projects/proj/sess.jsonl");
    fs::create_dir_all(inside.parent().unwrap()).unwrap();
    fs::write(&inside, b"x").unwrap();
    fs::write(td.path().join("outside.jsonl"), b"x").unwrap();
    // Construct a path that exits the sandbox via `..` segments.
    let traversal = td
        .path()
        .join("claude_projects/proj/../../outside.jsonl");
    assert!(sb.contains(&traversal).is_err());
}

#[test]
#[cfg(unix)]
fn it_sandbox_rejects_symlink_to_outside() {
    let (td, sb) = temp_sandbox();
    let outside = TempDir::new().unwrap();
    let target = outside.path().join("secret.txt");
    fs::write(&target, b"secret").unwrap();
    let link = td.path().join("claude_projects/escape.jsonl");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    // Canonicalization resolves the symlink to the outside target, which
    // then fails the starts_with(root) containment check.
    assert!(
        sb.contains(&link).is_err(),
        "symlink to outside must be rejected after canonicalize"
    );
}

#[test]
#[cfg(unix)]
fn it_sandbox_accepts_symlink_to_inside() {
    let (td, sb) = temp_sandbox();
    let real = td.path().join("claude_projects/real/foo.jsonl");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, b"x").unwrap();
    let link = td.path().join("claude_projects/alias/foo.jsonl");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let canon = sb
        .contains(&link)
        .expect("internal symlink must resolve to an in-sandbox target");
    assert!(canon.starts_with(sb.root()));
}

#[test]
#[cfg(unix)]
fn it_sandbox_rejects_nul_byte() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let (_td, sb) = temp_sandbox();
    let p = PathBuf::from(OsString::from_vec(b"foo\0bar.jsonl".to_vec()));
    let err = sb.contains(&p).expect_err("NUL byte must be rejected");
    assert!(format!("{err}").contains("NUL"), "error was: {err}");
}

// ---------------------------------------------------------------------------
// $HOME-dependent tests (ignored by default; run with `--ignored`).
// ---------------------------------------------------------------------------

/// Verifies that on a real developer machine where `~/.claude/projects`
/// exists, `validate_session_path` correctly accepts `/etc/passwd` as
/// outside the sandbox. Ignored by default because it touches the real
/// home directory and CI runners typically lack `~/.claude/projects`.
#[test]
#[ignore = "requires real ~/.claude/projects directory"]
fn it_validate_session_path_rejects_etc_passwd_on_real_home() {
    let err = validate_session_path(std::path::Path::new("/etc/passwd"))
        .expect_err("/etc/passwd must be rejected by the public API");
    assert!(format!("{err}").contains("outside sandbox"));
}

/// Verifies that on a real developer machine where `~/.claude/projects`
/// exists, `SandboxRoot::from_env()` constructs cleanly. Ignored because
/// it depends on the developer having that directory.
#[test]
#[ignore = "requires real ~/.claude/projects directory"]
fn it_sandbox_from_env_succeeds_on_real_home() {
    let sb = SandboxRoot::from_env().expect("real ~/.claude/projects exists");
    assert!(sb.root().is_absolute());
}

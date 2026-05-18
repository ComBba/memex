# Phase 1 — Security Hardening · SDD

**Phase ID**: P1
**KICKs**: KF-01 (SEC-003 path containment), KF-02 (SEC-004 snapshot path validation), KF-03 (signed snapshot envelope)
**Owner**: B2 Aisha (critic) — hold권 행사 가능
**Dependency**: 없음 (ship-blocker, 가장 먼저 실행)
**Day**: D-14
**Cross-phase invariants 적용**: 6개 invariant 모두 (특히 No-LLM, ComBba/memex remote)

---

## 1. KF-01 · SEC-003 Path Containment

### 1.1 Behavior contract

`get_session_turns` (Tauri command) 및 내부 호출 `parse_session(source_path)`은 다음을 보장한다:

1. `source_path`는 **`~/.claude/projects/`의 하위 경로**여야 한다 (정규화 후).
2. **Symlink는 canonicalize 후 검증** — symlink가 sandbox 밖을 가리키면 거부.
3. NUL byte (`\0`) 포함 경로 거부.
4. 검증 실패 시 `Err("path outside sandbox: {path}")` 반환 (절대 panic 금지).

### 1.2 API signature

```rust
// src-tauri/src/indexer/snapshot.rs (or shared sec.rs)
pub fn validate_session_path(path: &Path) -> anyhow::Result<PathBuf> {
    // returns canonical path if valid, Err otherwise
}
```

호출처:
- `commands.rs:get_session_turns` — 최초 IPC 입구
- `indexer.rs:1268`, `indexer.rs:1342` — predict_next_actions의 pivot 재파싱
- 모든 `source_path` 사용처 (analysis/04 SEC-003 인용)

### 1.3 Data structure

```rust
struct SandboxRoot {
    canonical: PathBuf,    // ~/.claude/projects canonical
}

impl SandboxRoot {
    pub fn from_env() -> anyhow::Result<Self> {
        let home = std::env::var("HOME")?;
        let root = PathBuf::from(home).join(".claude/projects");
        Ok(Self { canonical: root.canonicalize()? })
    }
    pub fn contains(&self, p: &Path) -> anyhow::Result<PathBuf> {
        let canon = p.canonicalize()?;
        if !canon.starts_with(&self.canonical) {
            anyhow::bail!("path outside sandbox: {}", canon.display());
        }
        Ok(canon)
    }
}
```

### 1.4 Edge cases (명세)

| 입력 | 기대 동작 |
|------|----------|
| `~/.claude/projects/abc/foo.jsonl` | OK (canonical 반환) |
| `~/.claude/projects/abc/../../etc/passwd` | Err (canonicalize → /etc/passwd, sandbox 밖) |
| `~/.claude/projects/abc/symlink-to-etc` (symlink) | Err (canonicalize 후 /etc/...) |
| `/etc/passwd` | Err (sandbox 밖) |
| `~/.claude/projects/abc/foo\0null.jsonl` | Err (NUL 포함) |
| 존재하지 않는 경로 | Err (canonicalize 실패) |
| 빈 문자열 | Err |

### 1.5 Failure mode · graceful degrade

- IPC 호출자 (frontend `main.js`)는 `Err`를 받으면 toast 알림: *"Session source path invalid (security check)"*.
- Replay surface는 해당 카드 disable.
- 다른 surface (Topology · Lens · Recall)는 영향 없음 (각자 자체 검증).

### 1.6 Acceptance criteria

- [ ] **AC-1.1.1** (owner: B3 Felix) `validate_session_path`가 분리된 함수로 존재
- [ ] **AC-1.1.2** (owner: B3 Felix) `commands.rs:get_session_turns` 진입 직후 호출
- [ ] **AC-1.1.3** (owner: B2 Aisha) `indexer.rs:1268`, `:1342`의 source_path 사용처 3개 모두 갱신
- [ ] **AC-1.1.4** (owner: B2 Aisha) Edge case 7개 모두 spec과 일치
- [ ] **AC-1.1.5** (owner: B2 Aisha) IPC error message가 frontend에 toast로 surface

---

## 2. KF-02 · SEC-004 Snapshot Path Validation

### 2.1 Behavior contract

`snapshot_export(path: PathBuf)` 및 `snapshot_import(path: PathBuf)`:

1. **app sandbox 하위만 허용** — macOS 기본 `~/Library/Application Support/dev.sgwannabe.memex/snapshots/`
2. 사용자 지정 경로가 sandbox 밖이면 거부.
3. 확장자 강제: `.snapshot` (또는 `.snapshot.zip`).
4. 디렉토리 traversal 거부 (`..`).

### 2.2 API signature

```rust
// src-tauri/src/indexer/snapshot.rs
pub struct SnapshotSandbox {
    root: PathBuf,    // ~/Library/Application Support/.../snapshots/
}

impl SnapshotSandbox {
    pub fn validate_path(&self, p: &Path, op: SnapshotOp) -> anyhow::Result<PathBuf> {
        // canonicalize, check parent within root, check extension
    }
}

pub enum SnapshotOp { Export, Import }
```

### 2.3 Data structure

```rust
const SNAPSHOT_EXT: &str = ".snapshot";

fn app_snapshot_dir() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let dir = PathBuf::from(home)
        .join("Library/Application Support/dev.sgwannabe.memex/snapshots");
    std::fs::create_dir_all(&dir)?;
    dir.canonicalize()
}
```

### 2.4 Edge cases

| 입력 | 동작 |
|------|------|
| `~/Library/.../snapshots/memex-20260518.snapshot` | OK |
| `~/Library/.../snapshots/../../../etc/foo` | Err (sandbox 밖) |
| `/tmp/foo.snapshot` | Err (sandbox 밖) |
| `~/Library/.../snapshots/foo.txt` | Err (ext 불일치) |
| Export 시 이미 존재하는 파일 | Err (overwrite 거부) |
| Import 시 존재 안 함 | Err |

### 2.5 Acceptance criteria

- [ ] **AC-1.2.1** (owner: B3 Felix) `SnapshotSandbox` 모듈 분리
- [ ] **AC-1.2.2** (owner: B3 Felix) snapshot_export/import 모두 validate_path 통과 후만 실행
- [ ] **AC-1.2.3** (owner: B2 Aisha) frontend는 native file picker 대신 sandbox 내 파일 리스트 prompt (post-MVP에 `tauri-plugin-dialog`)

---

## 3. KF-03 · Signed Snapshot Envelope

### 3.1 Behavior contract

Snapshot 파일은 외부 wrapper에 다음 메타데이터를 첨부한다:

```json
{
  "qdrant_snapshot": "<base64 or external file ref>",
  "sig": {
    "sha256": "<hex sha256 of qdrant_snapshot>",
    "issued_by": "memex/0.2.0",
    "issued_at": "2026-05-25T10:00:00Z",
    "schema_version": 3,
    "qdrant_version": "1.18.0"
  }
}
```

Import 시:
1. SHA-256 재계산 → `sig.sha256`과 비교, 불일치면 거부.
2. `schema_version`이 현 expected와 일치하지 않으면 경고 (자동 migration X, 사용자 confirm 후).
3. `qdrant_version`이 minor 불일치하면 경고.

### 3.2 API signature

```rust
pub struct SignedEnvelope {
    pub qdrant_snapshot_path: PathBuf,
    pub sig: Signature,
}

pub struct Signature {
    pub sha256: String,
    pub issued_by: String,
    pub issued_at: chrono::DateTime<Utc>,
    pub schema_version: u32,
    pub qdrant_version: String,
}

impl SignedEnvelope {
    pub fn sign(snapshot_path: &Path) -> anyhow::Result<Self> { ... }
    pub fn verify(&self) -> anyhow::Result<()> { ... }
}
```

### 3.3 Edge cases

| 케이스 | 동작 |
|------|------|
| 올바른 envelope | OK |
| sha256 mismatch (tamper) | Err |
| schema_version 불일치 | Warn + user confirm UI |
| qdrant_version minor 불일치 | Warn + import |
| qdrant_version major 불일치 | Err (1.x → 2.x) |
| envelope JSON malformed | Err |
| sig 필드 누락 | Err |

### 3.4 Acceptance criteria

- [ ] **AC-1.3.1** (owner: B3 Felix) `SignedEnvelope::sign` 으로 export 시 .sig 동봉
- [ ] **AC-1.3.2** (owner: B3 Felix) `SignedEnvelope::verify` 가 import 시 호출
- [ ] **AC-1.3.3** (owner: B2 Aisha) 위변조 snapshot import 거부 (test로 검증)
- [ ] **AC-1.3.4** (owner: A3 Liana) schema/qdrant version 불일치 시 UI confirm dialog

---

## 4. Phase 1 종합 acceptance

다음이 모두 ✅이면 P1 종료:

| ID | 항목 | 검증 방법 |
|----|------|----------|
| P1-DONE-1 | `validate_session_path` · `SnapshotSandbox` · `SignedEnvelope` 모듈 모두 컴파일 | `cargo check` |
| P1-DONE-2 | analysis/04의 SEC-003/004 finding이 close | tests.md의 모든 unit test 통과 |
| P1-DONE-3 | Replay/Snapshot surface가 깨지지 않음 | smoke test (G6) |
| P1-DONE-4 | 변경 파일이 모두 commit | `git log -1 --name-only` |
| P1-DONE-5 | 새 모듈이 `indexer/` 하위에 위치 (P3의 모듈 split 대비) | 디렉토리 확인 |

---

## 5. Risk · 이 phase가 실패할 경우

| Risk | Mitigation |
|------|------------|
| canonicalize 호출이 macOS sandbox에서 실패 (Full Disk Access 권한 부족) | 명시적 error message + README 안내 강화 |
| signed envelope SHA-256 호출이 큰 파일에서 느림 | streaming hash (`Sha256::new + update`) |
| 기존 snapshot 파일과 호환 안 됨 | envelope 미포함 시 "legacy mode" import 허용 (단 경고) |

---

## 6. Out-of-scope (P1에서 안 하는 것)

- 코드 서명 (CLA license)
- 다중 사용자 access control
- snapshot encryption (P3+ 고려)
- Tauri capability 재정의 (별도 phase)

P1은 **path containment + tampering detection** 두 가지에만 집중.

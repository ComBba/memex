# Phase 1 — Security Hardening · TDD

**Phase**: P1
**Tests precede implementation** (G2 gate).
**Test framework**: Rust `#[cfg(test)]` for unit, `cargo test --test integration` for integration.

---

## 1. Unit tests · KF-01 path containment

### Test file: `src-tauri/src/indexer/sec.rs` (`#[cfg(test)] mod tests`)

| Test name | Input | Expected | Maps to AC |
|-----------|-------|----------|-----------|
| `t_valid_session_path` | `~/.claude/projects/abc/sess.jsonl` (실 fixture) | `Ok(canonical_path)` | AC-1.1.1 |
| `t_path_outside_sandbox` | `/etc/passwd` | `Err(_)` | AC-1.1.4 |
| `t_path_traversal_dotdot` | `~/.claude/projects/abc/../../etc/passwd` | `Err(_)` (canonicalize → /etc/passwd) | AC-1.1.4 |
| `t_symlink_outside` | `~/.claude/projects/abc/link → /tmp/foo` (symlink fixture) | `Err(_)` | AC-1.1.4 |
| `t_symlink_inside` | `~/.claude/projects/abc/link → ~/.claude/projects/def/foo.jsonl` | `Ok(_)` | AC-1.1.4 |
| `t_nul_byte_path` | `~/.claude/projects/abc/foo\0bar.jsonl` | `Err(_)` | AC-1.1.4 |
| `t_empty_string` | `""` | `Err(_)` | AC-1.1.4 |
| `t_nonexistent_path` | `~/.claude/projects/nonexistent.jsonl` | `Err(_)` (canonicalize 실패) | AC-1.1.4 |

### Property tests (using `proptest` or hand-written)

| Property | Statement |
|----------|-----------|
| `prop_canonical_idempotent` | `validate(p).map(|c| validate(&c)) == validate(p)` 모든 valid p에 대해 |
| `prop_no_panic` | 임의 byte sequence 입력에서 panic 없음 |
| `prop_sandbox_invariant` | valid `p`는 항상 `~/.claude/projects` prefix를 가진다 |

---

## 2. Unit tests · KF-02 snapshot path validation

### Test file: `src-tauri/src/indexer/snapshot.rs` (`#[cfg(test)] mod path_tests`)

| Test name | Input | Expected |
|-----------|-------|----------|
| `t_export_valid_in_sandbox` | `~/Library/.../memex-export.snapshot`, Op::Export | `Ok(_)` |
| `t_export_outside_sandbox` | `/tmp/foo.snapshot`, Op::Export | `Err(_)` |
| `t_export_wrong_extension` | `~/Library/.../foo.txt`, Op::Export | `Err(_)` |
| `t_import_valid` | 존재하는 valid envelope, Op::Import | `Ok(_)` |
| `t_import_nonexistent` | 존재 안 하는 경로, Op::Import | `Err(_)` |
| `t_export_overwrites_existing` | 이미 존재하는 파일, Op::Export | `Err("already exists")` |
| `t_traversal_in_filename` | `~/Library/.../../../foo.snapshot` | `Err(_)` |

---

## 3. Unit tests · KF-03 signed snapshot envelope

### Test file: `src-tauri/src/indexer/snapshot.rs` (`#[cfg(test)] mod envelope_tests`)

| Test name | Setup | Action | Expected |
|-----------|-------|--------|----------|
| `t_sign_then_verify_ok` | tempdir에 fake snapshot blob 생성 | `sign` → `verify` | `Ok(())` |
| `t_verify_tampered_blob` | sign 후 blob 1 byte 수정 | `verify` | `Err("sha256 mismatch")` |
| `t_verify_tampered_sig_sha` | sign 후 .sig의 sha256 필드 수정 | `verify` | `Err("sha256 mismatch")` |
| `t_verify_missing_sig` | .sig 파일 삭제 | `verify` | `Err("no signature")` |
| `t_verify_legacy_no_envelope` | legacy snapshot (envelope 없음) | `verify` (legacy mode) | `Warn("legacy") + Ok(())` |
| `t_verify_schema_mismatch` | sig.schema_version = 1, current expected = 3 | `verify` | `Warn("schema") + Ok(())` (require user confirm) |
| `t_verify_qdrant_major_mismatch` | sig.qdrant_version = "2.0", current = "1.18" | `verify` | `Err("qdrant major mismatch")` |
| `t_verify_qdrant_minor_mismatch` | sig.qdrant_version = "1.17", current = "1.18" | `verify` | `Warn("minor") + Ok(())` |
| `t_envelope_json_malformed` | sig 파일이 invalid JSON | `verify` | `Err("malformed")` |

### Property tests

- `prop_sign_verify_roundtrip`: 임의 blob → sign → verify → Ok
- `prop_byte_flip_detected`: blob의 임의 1 byte flip → verify 항상 Err

---

## 4. Integration tests · IPC end-to-end

### Test file: `src-tauri/tests/sec_integration.rs`

| Test name | Action | Expected |
|-----------|--------|----------|
| `it_get_session_turns_valid` | Tauri command 호출 with valid session_id | Ok with turn list |
| `it_get_session_turns_tampered_payload` | Qdrant payload의 source_path를 `/etc/passwd`로 변조 후 호출 | Err message surface to IPC |
| `it_snapshot_export_then_import` | export → import roundtrip | 동일 collection state |
| `it_snapshot_import_tampered` | export → blob 변조 → import | Err message |

---

## 5. Regression tests · 기존 surface 보호

| Surface | Check | 기대 |
|---------|-------|------|
| Time Machine stack | `list_sessions` 호출 | 변화 없음 (sec 변경은 source_path 사용처만) |
| Topology | `topology` 호출 | 변화 없음 |
| Lens search | `lens_search` 호출 | 변화 없음 (path는 vector 검색에만 사용, file open X) |
| Mix & Match | `mix_match` 호출 | 변화 없음 |
| Proactive Recall | `tail_recent_errors` + `recall` | 변화 없음 (FS scan은 walkdir 별도 path) |
| Predict | `predict_next_actions` | source_path 사용처가 sec 통과해야 함 — 이 phase의 영향 |
| Replay | `get_session_turns` | sec 통과해야 함 — 이 phase의 영향 |

### Manual smoke checklist

- [ ] G6.1 `npm run tauri dev` → 첫 화면 Time Machine 정상 로드
- [ ] G6.2 카드 클릭 → Replay 패널 열림 (valid source_path)
- [ ] G6.3 Snapshot Export 버튼 → app sandbox에 저장
- [ ] G6.4 Snapshot Import 버튼 → 위에서 export 한 파일 선택 → 성공
- [ ] G6.5 macOS 키체인이나 외부 디렉토리 import 시도 → 에러 toast

---

## 6. Test fixture requirements

다음 fixture 파일을 `src-tauri/tests/fixtures/sec/` 에 추가 (P1 작업 중):

```
sec/
├── valid-session.jsonl           # ~/.claude/projects 모방용
├── valid-symlink-internal/        # 안전한 symlink
├── unsafe-symlink-external/       # /tmp 가리키는 symlink (CI에서 동적 생성)
├── tampered.snapshot              # 변조된 envelope
├── legacy.snapshot                # envelope 없는 legacy
└── README.md                      # fixture 설명
```

기존 fixture (`src-tauri/tests/fixtures/01_minimal.jsonl` 등)는 재사용 가능.

---

## 7. CI integration

`cargo test --manifest-path src-tauri/Cargo.toml` 1회 실행에 다음 모두 포함:

```
test indexer::sec::tests::t_valid_session_path ... ok
test indexer::sec::tests::t_path_outside_sandbox ... ok
test indexer::sec::tests::t_path_traversal_dotdot ... ok
test indexer::sec::tests::t_symlink_outside ... ok
test indexer::snapshot::path_tests::t_export_valid_in_sandbox ... ok
test indexer::snapshot::envelope_tests::t_sign_then_verify_ok ... ok
test indexer::snapshot::envelope_tests::t_verify_tampered_blob ... ok
... (총 24 unit + 4 integration = 28 tests)
```

전체 통과가 P1-DONE-2의 정의.

---

## 8. Test → AC mapping 요약

| Acceptance Criterion | Test 수 | Coverage |
|---------------------|---------|----------|
| AC-1.1.1 ~ AC-1.1.5 | 11 (8 unit + 3 prop) | path containment |
| AC-1.2.1 ~ AC-1.2.3 | 7 unit | snapshot path |
| AC-1.3.1 ~ AC-1.3.4 | 11 (9 unit + 2 prop) | envelope |
| Integration | 4 | IPC roundtrip |
| Regression | 5 smoke | 기존 7 surface 중 변경 영향 받는 2개 |

**총 38 test case**. 모두 spec.md AC와 1:1 mapping.

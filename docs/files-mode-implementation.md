# Files Mode — Implementation Reference

`src/modes/files.rs` — production-grade filesystem search for LaTUI.

---

## Overview

Files mode lets users locate and open files and directories from within the
launcher.  It mirrors the design language used by the **Run mode** and **Apps
mode** — the `Mode` trait is implemented in full, including `preview()`, and
all state is self-contained inside `FilesMode`.

---

## Key Features

| Feature | Detail |
|---|---|
| **Recent files** | Up to 200 entries, ranked by frequency + recency, persisted to `~/.local/share/latui/files_recents.json` |
| **Live directory walk** | `walkdir`-based, depth 5, covering `$HOME` (or custom roots) |
| **Fuzzy search** | Reuses the project's shared `SearchEngine` for recents matching |
| **Score merging** | Recents and walk results are merged and de-duplicated before returning |
| **Preview** | Text files → first 20 lines; Directories → child count; Symlinks → target path |
| **Security** | Path validation (length, null-bytes, existence), symlink traversal prevention, path-traversal guard |
| **Rate limiting** | 500 ms guard on `execute()`, 200 ms on `record_selection()` |
| **Persistence** | Recents saved on explicit `execute()` and on `Drop` |
| **Tests** | 14 unit tests covering creation, recents, search, preview, security, metadata round-trip |

---

## Architecture

```
FilesMode
├── recents: VecDeque<RecentEntry>           ← in-memory recents list
├── searchable_recents: Vec<SearchableItem>  ← indexed view of recents
├── search_engine: SearchEngine              ← shared fuzzy engine
├── search_roots: Vec<PathBuf>              ← directories to walk
├── recents_path: Option<PathBuf>           ← XDG data file location
├── last_action_time: Option<Instant>       ← rate-limit gate
└── dirty: bool                             ← pending-save flag
```

---

## Data Flow

```
User types query
      │
      ▼
Mode::search(query)
      │
      ├─ query.is_empty()? ──► get_recent_items()   →  VecDeque scored by
      │                                                 recency + open_count
      │
      └─ query.len() >= 1  ──► search_engine.search_scored()  (recents)
                           └─ query.len() >= 2  ──► walk_search()  (live FS)
                                                         │
                                                    merge + dedup by item.id
                                                         │
                                                    sort by score (desc)
                                                         │
                                                    take(60)

User selects item → Mode::execute(item)
      │
      ├─ parse FileMetadata JSON from item.metadata
      ├─ validate_path()  (existence / length / null bytes)
      ├─ Command::new("xdg-open").arg(path).spawn()
      ├─ add_to_recents()
      └─ save_recents()
```

---

## Metadata Format

Every `Item.metadata` field is JSON-encoded `FileMetadata`:

```json
{
  "path": "/home/user/Documents/notes.txt",
  "kind": "file"
}
```

`kind` is one of `"file"`, `"dir"`, or `"symlink"`.

This format is intentionally minimal.  The owning mode is the only consumer
of `metadata`, so adding fields later is non-breaking.

---

## Search Strategy

### Empty query
Returns the recent-files list (up to `RECENT_DISPLAY_LIMIT = 30`), sorted by:

```
score = (position_from_front × 10) + ln(open_count) × 15
```

Only paths that still exist on disk are included (stale entries are silently
filtered in `get_recent_items()`).

### Non-empty query
Two sources are combined:

1. **Recents fuzzy search** — `SearchEngine::search_scored()` over
   `searchable_recents`.  Each hit gets a +50 bonus to prefer already-known
   files over fresh walk results.

2. **Live directory walk** (query length ≥ 2) — `WalkDir` with
   `max_depth = 5`, sorted by filename match quality:

   | Match type | Score |
   |---|---|
   | Exact filename match | 1 000 |
   | Filename starts with query | 500 |
   | Filename contains query | 200 |
   | Directory (bonus) | +20 |

Results from both sources are merged using a `HashSet<id>` to avoid
duplicates, then sorted globally and capped at 60 items.

---

## Recents Persistence

```
XDG data dir / latui / files_recents.json
```

Structure on disk (pretty-printed for readability here):

```json
[
  {
    "path": "/home/user/notes.txt",
    "timestamp": 1742224340,
    "open_count": 7
  },
  ...
]
```

- Capped at **200 entries**.
- Stale paths (no longer existing on disk) are pruned on load.
- Files > 2 MiB are discarded (corruption guard).
- UNIX permissions are set to `0600` (owner read/write only).
- The containing directory is `0700`.

---

## Security

| Concern | Mitigation |
|---|---|
| **Path traversal** | Live-walk results are canonicalised and verified to start with the declared root |
| **Symlink follow** | `WalkDir::follow_links(false)` — symlinks are shown as items but never followed during the walk |
| **Null-byte injection** | `validate_path()` rejects paths containing `\0` |
| **Oversized paths** | Paths > 4 096 bytes are rejected |
| **Stale paths** | `validate_path()` checks existence before `xdg-open` |
| **Hidden dirs** | Directories whose name starts with `.` are skipped during the walk (depth > 0) |
| **Permission footprint** | Recents file and data directory are restricted to `0600` / `0700` |

---

## Preview

`FilesMode` sets `supports_preview() → true`.

| Kind | Preview content |
|---|---|
| Text file (≤ 512 KiB, ≤ 30 % non-printable bytes) | First 20 lines of content |
| Binary file | `"Binary file — N bytes"` |
| Directory | `"📁  Directory with N items"` |
| Symlink | `"🔗  Symlink → <target>"` |
| Gone (path deleted) | `"⚠️  File no longer exists"` |

Binary detection uses a simple heuristic: if more than 30 % of the first 512
bytes are control characters outside the normal text range, the file is
treated as binary.

---

## Configuration / Extension Points

### Custom search roots

```rust
// In main.rs — replace the FilesMode::new() call:
app.mode_registry.register(
    "files",
    Box::new(FilesMode::with_roots(vec![
        PathBuf::from("/home/user"),
        PathBuf::from("/mnt/nas/docs"),
    ])),
);
```

### Future: config file integration

The `search_paths` key in `config.toml` (defined in
`modes-implementation-docs.md`) can be wired into `FilesMode::with_roots()`
once the config loader is extended:

```toml
[modes.files]
search_paths = ["~", "~/Documents", "/mnt/nas"]
max_depth    = 4         # override SEARCH_MAX_DEPTH
```

### Future: file-type filtering

A `filter_extensions: Vec<String>` field can be added to `FilesMode` and
checked inside `walk_search()` before pushing a result.

---

## Constants

| Constant | Value | Purpose |
|---|---|---|
| `MAX_RECENTS` | 200 | Maximum entries in the recents list |
| `RECENT_DISPLAY_LIMIT` | 30 | Items shown for empty query |
| `SEARCH_MAX_DEPTH` | 5 | Maximum walkdir depth per root |
| `SEARCH_RESULT_LIMIT` | 60 | Maximum merged results returned |
| `PREVIEW_MAX_BYTES` | 524 288 | Max file size attempted for text preview |
| `PREVIEW_MAX_LINES` | 20 | Max preview lines shown |

---

## Unit Tests

All 14 tests live in `modes::files::tests` and run with:

```sh
cargo test --lib files
```

| Test | What it covers |
|---|---|
| `test_files_mode_creation` | Correct name / icon / empty state |
| `test_add_to_recents` | Single entry added correctly |
| `test_duplicate_recent_increments_count` | Duplicate promotes + bumps counter |
| `test_recent_promoted_to_front` | Re-opened entry moves to index 0 |
| `test_recents_size_capped` | Overflow keeps list at `MAX_RECENTS` |
| `test_validate_empty_path` | Empty path rejected |
| `test_validate_null_bytes` | Null-byte path rejected |
| `test_validate_too_long_path` | 5 000-char path rejected |
| `test_search_empty_returns_recents` | Empty query yields recent items |
| `test_search_with_query_uses_engine` | `walkdir` finds file in temp dir |
| `test_preview_text_file` | Multi-line text preview correct |
| `test_preview_directory` | Directory item count correct |
| `test_filekind_icons` | Icon constants correct |
| `test_metadata_roundtrip` | JSON encode → decode is lossless |

---

## Files Changed

| File | Change |
|---|---|
| `src/modes/files.rs` | **Full implementation** (replaces stub) |
| `Cargo.toml` | Added `tempfile = "3"` to `[dev-dependencies]` |

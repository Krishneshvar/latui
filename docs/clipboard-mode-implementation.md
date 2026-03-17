# Clipboard Mode — Implementation Reference

`src/modes/clipboard.rs` — searchable clipboard history manager for LaTUI.

---

## Overview

Clipboard mode gives users a fast, fuzzy-searchable view of their clipboard
history.  Selecting any entry immediately writes it back to the system clipboard
so it is ready to paste anywhere.  History survives restarts via a JSON file in
the XDG data directory.

---

## Key Features

| Feature | Detail |
|---|---|
| **History** | Up to 500 entries, capped + de-duplicated, promoted-front on reuse |
| **Persistence** | `~/.local/share/latui/clipboard_history.json`, secured to `0600` |
| **Fuzzy search** | Full-content + title fuzzy matching via the shared `SearchEngine` |
| **Preview** | Full clip content, truncated at 20 lines with a "N more lines" indicator |
| **Backend detection** | Wayland-first (`wl-copy`), X11 fallback (`xclip`), graceful no-op otherwise |
| **Paste** | Content piped to stdin of the clipboard tool — no shell injection possible |
| **Rate limiting** | 500 ms gate on `execute()`, 200 ms on `record_selection()` |
| **Privacy** | Actual clip content is **never logged**; only lengths and counts appear in trace |
| **Tests** | 21 unit tests covering creation, dedup, capping, title truncation, search, preview, validation, backend |

---

## Architecture

```
ClipboardMode
├── history: VecDeque<ClipEntry>       ← in-memory history (most recent first)
├── searchable: Vec<SearchableItem>    ← fuzzy-indexed view of history
├── search_engine: SearchEngine        ← shared fuzzy engine
├── backend: ClipBackend               ← Wayland | X11 | None
├── history_path: Option<PathBuf>      ← XDG data file location
├── last_action_time: Option<Instant>  ← rate-limit gate
└── dirty: bool                        ← pending-save flag
```

---

## Data Flow

```
User types query
      │
      ▼
Mode::search(query)
      │
      ├─ query.is_empty()? ──► get_recent_items()
      │                          scored by recency + ln(use_count)
      │
      └─ non-empty query  ──► SearchEngine::search_scored()
                               over searchable (content + title fields)

User selects entry → Mode::execute(item)
      │
      ├─ read content from item.metadata
      ├─ validate_content() — empty / oversized check
      ├─ write_clipboard()
      │      └─ wl-copy (Wayland) or xclip (X11)  ← piped stdin
      ├─ record_clip()  — promote in history
      └─ save_history()
```

---

## Backend Detection

Backend is resolved once at construction time:

```
$WAYLAND_DISPLAY set  AND  `wl-copy` in PATH  →  Wayland
$DISPLAY set          AND  `xclip` in PATH    →  X11
otherwise                                      →  None (warns on execute)
```

On this machine the active backend is **Wayland** (`wl-copy`).

---

## History Entry Format

Persisted as a JSON array of `ClipEntry` objects:

```json
[
  {
    "content": "some text the user copied",
    "first_seen": 1742224000,
    "last_used":  1742227340,
    "use_count":  3
  },
  ...
]
```

- Capped at **500 entries**.
- Entries with empty content or content > 64 KiB are filtered on load.
- File size > 8 MiB causes the entire file to be discarded (corruption guard).
- UNIX file permissions: `0600`; parent directory: `0700`.

---

## Item Metadata Format

Unlike Files mode, metadata is the **raw content** string directly:

```
item.metadata = Some("the copied text")
```

This is intentionally simple — the content is both the display source and the
paste payload.  No wrapping struct needed.

---

## Display Title Rules

The title shown in the results list is derived from `make_title(content)`:

1. Take the **first non-empty line** of the content.
2. Truncate to **80 characters**.
3. Append `…` if truncated.
4. Append `⏎` if the clip spans multiple lines.

Examples:

| Content | Title |
|---|---|
| `"hello world"` | `hello world` |
| `"foo\nbar\nbaz"` | `foo ⏎` |
| `"a".repeat(100)` | `"aaaa…80chars…aaaa…"` |
| `"\n\n  real text"` | `real text` |

---

## Search Strategy

### Empty query
Returns the most recent `min(len, 30)` entries sorted by:

```
score = (position_from_front × 10) + ln(use_count + 1) × 15
```

### Non-empty query
The shared `SearchEngine` fuzzy-matches over two `SearchableItem` fields:

| Field | Weight | Purpose |
|---|---|---|
| `"title"` | 8.0 | First line / truncated display string |
| `"content"` | 5.0 | Full text of the clipboard entry |

Higher title weight means first-line matches bubble up, but multi-line clips
whose relevant content is deeper in the text are still discoverable.

---

## Preview

`ClipboardMode` sets `supports_preview() → true`.

| Content length | Preview |
|---|---|
| ≤ 20 lines | Full content verbatim |
| > 20 lines | First 20 lines + `"… N more lines"` trailer |
| `metadata = None` | `None` (no preview panel shown) |

Actual content is displayed without sanitisation in the preview — it is the
user's own data.

---

## Security & Privacy

| Concern | Mitigation |
|---|---|
| **Log leakage** | Content is never logged; only `len`, `use_count`, and `title` appear at trace level |
| **Shell injection** | Content is piped to `wl-copy`/`xclip` via **stdin** — never passed as a CLI argument |
| **Oversized payloads** | `MAX_CONTENT_BYTES = 64 KiB` — larger content is silently ignored |
| **File permissions** | `0600` on history JSON, `0700` on data directory |
| **Corrupt files** | JSON parse failures and files > 8 MiB are silently discarded |

---

## Extension Points

### Custom history limit
Change `MAX_HISTORY` at the top of `clipboard.rs`:

```rust
const MAX_HISTORY: usize = 1000; // keep more history
```

### Image / binary clipboard support
The current implementation is text-only.  Binary support would require:
1. A `kind: "text" | "image"` discriminator in the stored entry.
2. Writing the binary blob with `wl-copy --type image/png` etc.
3. A separate image preview renderer in the UI.

### Automatic background polling
LaTUI does not currently poll the system clipboard automatically.  To capture
entries from other applications, a background thread could call `wl-paste
--watch` and call `record_clip()` on each new line received.  This would
require exposing a thread-safe interface (e.g., `Arc<Mutex<ClipboardMode>>`).

---

## Constants

| Constant | Value | Purpose |
|---|---|---|
| `MAX_HISTORY` | 500 | Maximum entries in history |
| `RECENT_DISPLAY_LIMIT` | 30 | Entries shown for empty query |
| `TITLE_MAX_CHARS` | 80 | Max chars in the display title |
| `MAX_CONTENT_BYTES` | 65 536 | Max accepted clip size |
| `MAX_HISTORY_FILE_BYTES` | 8 388 608 | Max JSON file size before discard |

---

## Unit Tests

All 21 tests live in `modes::clipboard::tests` and run with:

```sh
cargo test --lib clipboard
```

| Test | What it covers |
|---|---|
| `test_creation` | Correct name / icon / empty state |
| `test_add_single_entry` | First clip stored correctly |
| `test_duplicate_promotes_and_increments` | Re-clip moves entry to front, bumps count |
| `test_history_capped_at_max` | Overflow held at `MAX_HISTORY` |
| `test_empty_content_ignored` | Empty string silently dropped |
| `test_oversized_content_ignored` | > 64 KiB silently dropped |
| `test_title_short_single_line` | Short single-line title unchanged |
| `test_title_multiline_shows_indicator` | `⏎` appended for multi-line |
| `test_title_truncated_at_max_chars` | 80-char truncation + `…` |
| `test_title_skips_leading_empty_lines` | First non-empty line selected |
| `test_validate_empty_fails` | Empty content rejected |
| `test_validate_oversized_fails` | Oversized content rejected |
| `test_validate_normal_passes` | Normal text accepted |
| `test_search_empty_returns_recent` | Most recent entry is first |
| `test_search_finds_substring` | Engine locates matching entry |
| `test_search_no_match_returns_empty` | No results for unmatched query |
| `test_preview_short_content` | Short content returned verbatim |
| `test_preview_truncates_long_content` | 30-line content truncated with trailer |
| `test_preview_none_when_no_metadata` | `None` when no metadata |
| `test_backend_name` | `name()` returns correct strings |
| `test_dirty_set_after_record` | `dirty` flag set after `record_clip` |

---

## Files Changed

| File | Change |
|---|---|
| `src/modes/clipboard.rs` | **Full implementation** (replaces stub) |

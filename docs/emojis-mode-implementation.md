# Emojis Mode — Implementation Reference

`src/modes/emojis.rs` — embedded emoji picker for LaTUI.

---

## Overview

Emojis mode provides instant access to a curated set of Unicode emojis.
Users search by name, keyword, or category; selecting an entry copies the
glyph directly to the system clipboard.  Recently used emojis are surfaced
first and their frequency is persisted across sessions.

---

## Key Features

| Feature | Detail |
|---|---|
| **Static database** | 240+ emojis, 8 categories, multi-keyword indexed |
| **Fuzzy search** | Name (weight 10), keywords (weight 8), category (weight 4) via shared `SearchEngine` |
| **Recency tracking** | Up to 100 recent picks, persisted to `~/.local/share/latui/emoji_recents.json` |
| **Empty query** | Shows recent/frequent picks first; falls back to top of static list |
| **Preview** | Large glyph + name, category, keywords, usage count |
| **Copy backend** | Wayland (`wl-copy`) preferred, X11 (`xclip`) fallback, stdin-piped |
| **Rate limiting** | 500 ms gate on `execute()` |
| **Tests** | 18 unit tests covering index, search, recents, preview, validation, backend |

---

## Emoji Database

The static database is a Rust `&[EmojiRow]` where each row is:

```rust
type EmojiRow = (&'static str, &'static str, &'static [&'static str], &'static str);
//               glyph          name           keywords                  category
```

Example entries:

```rust
("😂", "face with tears of joy", &["lol","laugh","tears","funny","haha"], "smileys"),
("🔥", "fire",                   &["fire","hot","flame","lit","burn"],     "nature"),
("💻", "laptop",                 &["laptop","computer","tech","coding"],    "objects"),
```

### Categories covered

| Category | Examples |
|---|---|
| `smileys` | 😀 😂 🤔 😈 🤖 |
| `people` | 👋 👍 🙏 💪 ✌️ |
| `nature` | 🌸 🔥 ❄️ 🌈 🌍 |
| `animals` | 🐶 🦁 🦋 🐝 🐧 |
| `food` | 🍕 🍣 ☕ 🍺 🎂 |
| `activities` | ⚽ 🎮 🏆 🎵 🎨 |
| `objects` | 💻 🔑 💡 📊 🔬 |
| `travel` | ✈️ 🚀 🏠 🏖️ 🌴 |
| `symbols` | ❤️ ✅ 🎉 💯 🔔 |

---

## Architecture

```
EmojisMode
├── searchable: Vec<SearchableItem>    ← indexed from EMOJIS at load time
├── recents: VecDeque<RecentEmoji>     ← most-recently-used (persisted)
├── search_engine: SearchEngine        ← shared fuzzy engine
├── backend: CopyBackend               ← Wayland | X11 | None
├── recents_path: Option<PathBuf>      ← XDG data file
├── last_action_time: Option<Instant>  ← rate-limit gate
└── dirty: bool                        ← pending-save flag
```

---

## Data Flow

```
load()
  └─ build_index()  →  iterate EMOJIS static slice
                         →  SearchableItem per emoji
                              name × 10, keywords × 8, category × 4

search(query)
  ├─ empty  →  get_recent_display()
  │              recent entries → lookup name in EMOJIS, score by recency+freq
  │              fallback → first RECENT_DISPLAY_LIMIT from EMOJIS
  └─ typed  →  SearchEngine::search_scored(query, &searchable)

execute(item)
  ├─ rate-limit check
  ├─ glyph = item.metadata
  ├─ copy_to_clipboard(glyph)   ← piped stdin to wl-copy / xclip
  ├─ record_use(glyph)          ← promote in recents
  └─ save_recents()
```

---

## Metadata Format

`Item.metadata` = the raw emoji glyph itself (e.g. `"🔥"`).

No JSON wrapper — the glyph is both the display character and the clipboard payload.

---

## Search Field Weights

| Field | Weight | Purpose |
|---|---|---|
| `name` | 10.0 | Primary search target — "fire", "pizza" |
| `keyword` | 8.0 | Synonyms — "hot", "flame", "burn" for 🔥 |
| `category` | 4.0 | Broad category match — "nature", "food" |

---

## Recency Persistence

```
XDG data dir / latui / emoji_recents.json
```

```json
[
  { "glyph": "😂", "use_count": 12, "last_used": 1742224340 },
  { "glyph": "🔥", "use_count":  5, "last_used": 1742220000 }
]
```

- Capped at **100 entries**.
- Files > 512 KiB are discarded.
- Permissions: `0600` / parent `0700`.

---

## Preview

```
😀

Name: grinning face
Category: smileys
Keywords: happy, smile, grin, joy
Used 3 times
```

Returns `None` for glyphs not found in the static database (no crash).

---

## Empty Query Behaviour

1. If recents exist → sort by `(position × 10) + ln(use_count+1) × 15`, return top 24.
2. If no recents → return first 24 rows from `EMOJIS` static slice.

This ensures the mode always shows something useful on first launch.

---

## Copy Backend

Identical detection logic to Clipboard mode:

```
$WAYLAND_DISPLAY set AND wl-copy in PATH  →  Wayland (stdin pipe)
$DISPLAY set         AND xclip in PATH    →  X11 (stdin pipe)
otherwise                                 →  None (error returned)
```

Content is always piped via **stdin**, never passed as a CLI argument.

---

## Extension Points

### Adding emojis
Append rows to the `EMOJIS` static slice — no other code change needed:

```rust
("🫶", "heart hands", &["love","heart","hands","care"], "people"),
```

### Loading emojis from a file
Replace `EMOJIS` with a runtime-loaded JSON/TOML file in `load()`.
The `searchable` Vec already separates data from logic.

### Category filtering
Add a `category_filter: Option<&'static str>` field and apply it in `search()`
before delegating to the engine — useful for a future category tab UI.

---

## Constants

| Constant | Value | Purpose |
|---|---|---|
| `MAX_RECENTS` | 100 | Max recent emoji entries |
| `RECENT_DISPLAY_LIMIT` | 24 | Entries shown for empty query |

---

## Unit Tests (18 total)

```sh
cargo test --lib emojis
```

| Test | Covers |
|---|---|
| `test_creation` | Name / icon / empty state |
| `test_build_index_populates_all` | All EMOJIS rows indexed |
| `test_search_by_name` | Name field match ("pizza" → 🍕) |
| `test_search_by_keyword` | Keyword match ("laugh" → 😂 / 🤣) |
| `test_search_by_category` | Category match ("travel") |
| `test_search_empty_returns_defaults` | Empty query not empty |
| `test_search_no_match` | Nonsense query returns empty |
| `test_record_use_adds_recent` | Single entry recorded |
| `test_record_use_increments_duplicate` | Re-use bumps count |
| `test_record_use_promotes_to_front` | Re-used entry moves to index 0 |
| `test_recents_capped` | Overflow held at MAX |
| `test_dirty_flag` | Flag set after record_use |
| `test_preview_known_emoji` | Preview contains name, category, keywords |
| `test_preview_unknown_emoji_returns_none` | Out-of-db glyph → None |
| `test_metadata_is_glyph` | item.metadata == raw glyph |
| `test_static_db_no_empty_entries` | No malformed rows in EMOJIS |
| `test_supports_preview` | supports_preview() == true |
| `test_backend_name_coverage` | Backend enum equality |

---

## Files Changed

| File | Change |
|---|---|
| `src/modes/emojis.rs` | **Full implementation** (replaces stub) |

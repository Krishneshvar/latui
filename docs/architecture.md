# LATUI Architecture Overview

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         LATUI TUI LAUNCHER                       │
│                         (Wayland/X11)                            │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                          MAIN LOOP                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  UI Render   │  │ Event Handle │  │ State Update │          │
│  │  (Ratatui)   │  │ (Crossterm)  │  │  (AppState)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                         MODE SYSTEM                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Apps Mode   │  │  Files Mode  │  │  Calc Mode   │          │
│  │  (Current)   │  │   (Future)   │  │   (Future)   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SEARCH ENGINE                               │
│  ┌──────────────────────────────────────────────────────┐       │
│  │                   Query Processing                    │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │       │
│  │  │Tokenizer │→ │  Parser  │→ │ Normaliz.│           │       │
│  │  └──────────┘  └──────────┘  └──────────┘           │       │
│  └──────────────────────────────────────────────────────┘       │
│                                                                   │
│  ┌──────────────────────────────────────────────────────┐       │
│  │                  Candidate Selection                  │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │       │
│  │  │   Trie   │→ │ Keyword  │→ │ Inverted │           │       │
│  │  │  Index   │  │  Mapper  │  │  Index   │           │       │
│  │  └──────────┘  └──────────┘  └──────────┘           │       │
│  └──────────────────────────────────────────────────────┘       │
│                                                                   │
│  ┌──────────────────────────────────────────────────────┐       │
│  │                   Hybrid Scoring                      │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │       │
│  │  │  Exact   │  │  Prefix  │  │   Word   │           │       │
│  │  │  Match   │  │  Match   │  │ Boundary │           │       │
│  │  └──────────┘  └──────────┘  └──────────┘           │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │       │
│  │  │ Acronym  │  │  Fuzzy   │  │ Keyword  │           │       │
│  │  │  Match   │  │  Match   │  │  Match   │           │       │
│  │  └──────────┘  └──────────┘  └──────────┘           │       │
│  │  ┌──────────┐                                        │       │
│  │  │   Typo   │                                        │       │
│  │  │Tolerance │                                        │       │
│  │  └──────────┘                                        │       │
│  └──────────────────────────────────────────────────────┘       │
│                                                                   │
│  ┌──────────────────────────────────────────────────────┐       │
│  │                    Final Ranking                      │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │       │
│  │  │Frequency │→ │ Recency  │→ │ Learning │           │       │
│  │  │  Boost   │  │  Boost   │  │  Boost   │           │       │
│  │  └──────────┘  └──────────┘  └──────────┘           │       │
│  └──────────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      DATA LAYER                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  App Cache   │  │   SQLite DB  │  │    Config    │          │
│  │   (JSON)     │  │  (Usage/Log) │  │    (TOML)    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module Dependency Graph

```
main.rs
  │
  ├─► app::state (AppState)
  │     └─► core::item (Item)
  │
  ├─► modes::apps (AppsMode)
  │     ├─► core::mode (Mode trait)
  │     ├─► core::item (Item)
  │     ├─► core::action (Action)
  │     ├─► cache::apps_cache (load/save)
  │     └─► search::engine (SearchEngine) ◄── NEW
  │
  ├─► search::engine (SearchEngine) ◄── NEW
  │     ├─► search::tokenizer (Tokenizer)
  │     ├─► search::scorer (HybridScorer)
  │     ├─► search::typo (TypoTolerance)
  │     ├─► search::ranker (Ranker)
  │     ├─► config::keywords (KeywordMapper)
  │     ├─► index::trie (Trie)
  │     └─► matcher::fuzzy (FuzzyMatcher)
  │
  ├─► tracking::frequency (FrequencyTracker) ◄── NEW
  │     └─► tracking::database (Database)
  │
  ├─► tracking::learning (LearningSystem) ◄── NEW
  │
  ├─► config::keywords (KeywordMapper) ◄── NEW
  │     └─► config::loader (load config)
  │
  └─► ui::renderer (draw)
        └─► app::state (AppState)
```

---

## Data Flow: Search Query

```
User Types "browser"
        │
        ▼
┌─────────────────┐
│   AppState      │  query = "browser"
│   (main.rs)     │
└─────────────────┘
        │
        ▼
┌─────────────────┐
│   AppsMode      │  search("browser")
│   (modes/apps)  │
└─────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────┐
│              SearchEngine                        │
│                                                  │
│  1. Tokenize: "browser" → ["browser"]           │
│                                                  │
│  2. Trie Lookup: Get candidates                 │
│     - "firefox" (no direct match)               │
│     - "chrome" (no direct match)                │
│     - "brave" (no direct match)                 │
│                                                  │
│  3. Keyword Match:                               │
│     - "browser" → ["firefox", "chrome", ...]    │
│     ✓ Firefox matches!                          │
│     ✓ Chrome matches!                           │
│     ✓ Brave matches!                            │
│                                                  │
│  4. Score Each Match:                            │
│     Firefox:                                     │
│       - Keyword match: 150                      │
│       - Field weight (keywords): ×8             │
│       - Base score: 1200                        │
│       - Frequency boost: +48 (10 launches)      │
│       - Recency boost: +50 (used 30 min ago)    │
│       - TOTAL: 1298                             │
│                                                  │
│     Chrome:                                      │
│       - Keyword match: 150                      │
│       - Field weight (generic_name): ×6         │
│       - Base score: 900                         │
│       - Frequency boost: +20 (3 launches)       │
│       - Recency boost: +0 (used 2 days ago)     │
│       - TOTAL: 920                              │
│                                                  │
│     Brave:                                       │
│       - Keyword match: 150                      │
│       - Field weight (categories): ×5           │
│       - Base score: 750                         │
│       - Frequency boost: +14 (2 launches)       │
│       - Recency boost: +30 (used 12 hours ago)  │
│       - TOTAL: 794                              │
│                                                  │
│  5. Sort by Score: [Firefox, Chrome, Brave]     │
│                                                  │
└─────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────┐
│   AppState      │  filtered_items = [Firefox, Chrome, Brave]
│   (main.rs)     │
└─────────────────┘
        │
        ▼
┌─────────────────┐
│   UI Renderer   │  Display results
│   (ui/renderer) │
└─────────────────┘
```

---

## Data Flow: App Launch

```
User Presses Enter on "Firefox"
        │
        ▼
┌─────────────────┐
│   main.rs       │  Get selected item
└─────────────────┘
        │
        ▼
┌─────────────────┐
│   AppsMode      │  execute(item)
│   (modes/apps)  │
└─────────────────┘
        │
        ├─► Launch app via Command::new()
        │
        └─► Record usage
              │
              ▼
        ┌─────────────────────┐
        │ FrequencyTracker    │  record_launch("firefox")
        │ (tracking/frequency)│
        └─────────────────────┘
              │
              ├─► Update launch_count: 10 → 11
              ├─► Update last_used: now()
              │
              ▼
        ┌─────────────────────┐
        │    Database         │  Save to SQLite
        │ (tracking/database) │
        └─────────────────────┘
```

---

## File Organization

```
src/
│
├── Core Types & Traits
│   ├── core/item.rs          - Item struct
│   ├── core/action.rs        - Action enum
│   └── core/mode.rs          - Mode trait
│
├── Application State
│   └── app/state.rs          - AppState (query, items, selection)
│
├── Search System ◄── NEW
│   ├── search/engine.rs      - Orchestrates search pipeline
│   ├── search/tokenizer.rs   - Text → tokens
│   ├── search/scorer.rs      - Multi-algorithm scoring
│   ├── search/typo.rs        - Levenshtein distance
│   └── search/ranker.rs      - Final ranking with boosts
│
├── Indexing
│   └── index/trie.rs         - Prefix tree for fast lookup
│
├── Matching
│   └── matcher/fuzzy.rs      - Nucleo fuzzy matcher wrapper
│
├── Configuration ◄── NEW
│   ├── config/keywords.rs    - Keyword → app mappings
│   ├── config/keywords.toml  - Default mappings
│   └── config/loader.rs      - Load user config
│
├── Usage Tracking ◄── NEW
│   ├── tracking/database.rs  - SQLite connection
│   ├── tracking/frequency.rs - Launch frequency stats
│   └── tracking/learning.rs  - Query → selection patterns
│
├── Modes
│   └── modes/apps.rs         - Application launcher mode
│
├── Caching
│   └── cache/apps_cache.rs   - JSON cache for apps
│
└── UI
    └── ui/renderer.rs        - Ratatui rendering
```

---

## Configuration Files

```
~/.config/latui/
├── keywords.toml          - User keyword mappings
└── config.toml            - General settings (future)

~/.cache/latui/
├── apps.json              - Cached application list
└── usage.db               - SQLite usage statistics
```

---

## Search Algorithm Flow

```
Query: "fir"
  │
  ├─► 1. TOKENIZE
  │     Input: "fir"
  │     Output: ["fir"]
  │
  ├─► 2. TRIE LOOKUP (Fast prefix filter)
  │     Input: "fir"
  │     Output: [Firefox, Firestorm, ...]
  │     Time: O(m) where m = query length
  │
  ├─► 3. KEYWORD MATCH (Semantic search)
  │     Input: "fir"
  │     Output: [] (no keyword matches)
  │
  ├─► 4. SCORE CANDIDATES (Parallel)
  │     For each candidate:
  │       ├─► Exact match?     → 1000 points
  │       ├─► Prefix match?    → 500 points  ✓ "fir" prefix of "firefox"
  │       ├─► Word boundary?   → 300 points
  │       ├─► Acronym?         → 250 points
  │       ├─► Fuzzy match?     → 0-200 points
  │       ├─► Keyword match?   → 150 points
  │       └─► Typo tolerance?  → 50-150 points
  │
  ├─► 5. APPLY FIELD WEIGHTS
  │     Score × field_weight
  │     - Name field: ×10
  │     - Keywords: ×8
  │     - Generic name: ×6
  │
  ├─► 6. ADD BOOSTS
  │     + Frequency boost (based on launch count)
  │     + Recency boost (based on last used)
  │     + Learning boost (based on query history)
  │
  └─► 7. SORT & RETURN
        Output: [Firefox (5500), Firestorm (5000), ...]
```

---

## Performance Characteristics

### Time Complexity
- **Trie lookup:** O(m) where m = query length
- **Fuzzy matching:** O(n × m) where n = candidates, m = query length
- **Sorting:** O(n log n) where n = matched items
- **Total:** O(m + n × m + n log n)

### Space Complexity
- **Trie:** O(k × l) where k = items, l = avg token length
- **Items:** O(k × f) where k = items, f = avg fields
- **Cache:** O(k) for previous results
- **Total:** O(k × (l + f))

### Expected Performance
- **Empty query:** < 1ms (return all)
- **Short query (1-2 chars):** < 5ms (trie + fuzzy)
- **Normal query (3-6 chars):** < 10ms (full pipeline)
- **Complex query (7+ chars):** < 15ms (with all boosts)

---

## Future Enhancements

### Additional Modes
```
modes/
├── apps.rs       ✓ Current
├── files.rs      ⚡ File search
├── calc.rs       ⚡ Calculator
├── clipboard.rs  ⚡ Clipboard history
├── windows.rs    ⚡ Window switcher
└── ssh.rs        ⚡ SSH hosts
```

### Plugin System
```
plugins/
├── api.rs        - Plugin trait
├── loader.rs     - Dynamic loading
└── registry.rs   - Plugin registry
```

### Advanced Search
```
search/
├── semantic.rs   - Semantic understanding
├── nlp.rs        - Natural language
├── predict.rs    - Predictive suggestions
└── cluster.rs    - App clustering
```

---

**Architecture Status:** ✅ Ready for Implementation

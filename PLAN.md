# Prompt Deduplicator - Project Plan

## Overview
A native GUI application for managing, deduplicating, and organizing image generation prompts. Built in Rust for performance and cross-platform support (macOS + Windows).

## Core Features

### 1. File Import
- Load multiple .txt files containing prompts (one prompt per line)
- Support drag & drop and file picker
- Display total prompts loaded, source files

### 2. Similarity Detection
- Find similar/duplicate prompts using text similarity algorithms
- Group similar prompts together for review
- Configurable similarity threshold

### 3. Deduplication Workflow
- View similar prompt groups side by side
- Select which prompt to keep
- Delete/remove others
- Batch operations

### 4. Search
- Keyword search (e.g., "pool" returns all prompts containing "pool")
- Real-time filtering as you type
- Result count

### 5. Mass Find & Replace
- Find text across all prompts
- Replace with new text
- Preview changes before applying
- Regex support (stretch goal)

### 6. Export
- Save cleaned prompts to file
- Preserve or merge into single file

## Tech Stack
- **Language**: Rust
- **GUI Framework**: egui (via eframe)
- **Similarity Algorithm**: TBD (start with Levenshtein/Jaccard, evaluate embeddings later)
- **Packaging**: Native binaries for macOS (.app) and Windows (.exe)

## Architecture

```
prompt-dedup/
├── src/
│   ├── main.rs          # Entry point, app initialization
│   ├── app.rs           # Main application state and UI
│   ├── prompt.rs        # Prompt data structures
│   ├── similarity.rs    # Similarity detection algorithms
│   ├── search.rs        # Search and filter logic
│   └── file_io.rs       # File loading and saving
├── Cargo.toml           # Dependencies
├── PLAN.md              # This file
├── TODO.md              # Progress tracking
└── NOTES.md             # Session notes, context
```

## UI Screens (Rough Plan)

1. **Home/Import**: Load files, see stats
2. **Browse**: View all prompts, search/filter
3. **Deduplicate**: Review similar groups, choose keepers
4. **Find & Replace**: Mass editing

## Open Questions
- Similarity algorithm: pure text vs embeddings?
- Threshold for "similar" — user configurable?
- How to handle very large files (100k+ lines)?

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-02 | Use Rust + egui | Performance, small binary, cross-platform |
| 2026-02-02 | Start with text similarity | No external dependencies, fast, upgrade later if needed |

---
*Last updated: 2026-02-02*

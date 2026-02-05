# Prompt Deduplicator - Session Notes

## Context for Future Sessions
When returning to this project after a break, read this file + PLAN.md + TODO.md to get caught up.

---

## Session 1 - 2026-02-02

### What happened
- Discussed project requirements
- Decided on Rust + egui for native cross-platform GUI
- Created project structure with `cargo new`
- Set up documentation files

### User context
- User is learning Rust through this project
- Has ~20k lines of image generation prompts across multiple files
- Wants to deduplicate similar prompts (e.g., "burgundy lingerie" vs "maroon lingerie")
- Needs: similarity detection, search, find/replace, clean GUI
- Must work on both macOS (dev machine) and Windows (home PC)

### Current state
- Fresh Rust project created
- No code written yet
- Next: add egui dependency and create basic window

### Key decisions
- Rust over Python for efficiency and small binary size
- egui for GUI (simple, cross-platform, pure Rust)
- Text-based similarity first (no ML models to start)

---

## Technical Notes

### Useful commands
```bash
cargo run          # Build and run
cargo build        # Build only
cargo build --release  # Optimized build
```

### egui resources
- Docs: https://docs.rs/egui
- Examples: https://github.com/emilk/egui/tree/master/examples

---
*Add new session notes above this line*

# CONTEXT - Read This First

## Workflow
1. Read docs, come up with a plan to execute the current step
2. User approves, edits, or rejects the plan
3. Make the changes
4. Verify it builds
5. User tests (if needed for the step)
6. Update docs and mark step as DONE

---

## Project
Rust GUI app to deduplicate image generation prompts (~20k lines).

## Tech
- Rust + egui (eframe)
- SQLite (rusqlite) for persistent storage
- Cross-platform: macOS + Windows

## Current State
- File import, search, scrollable list working
- Catppuccin Macchiato theme, card layout, search highlighting, copy buttons
- Tab-based UI (Browse / Deduplicate)
- Similarity detection with Jaccard algorithm, side-by-side comparison, delete/skip, batch remove
- Find & Replace popup (Cmd/Ctrl+R) with live preview
- Export to .txt with file dialog
- Status bar with success/error messages (auto-clears after 5s)

## Last Session
2026-02-05: Completed Milestone 7-8. macOS release binary built. GitHub Actions workflow for cross-platform builds (macOS arm64/x64 + Windows).

---
*Update this file at end of each session. Keep it SHORT.*

# Upload State Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace fragile Tauri upload event delivery with persisted backend upload state that the frontend can safely read after the window is shown.

**Architecture:** Add a Rust upload tracker to `AppState` as the source of truth, update the IPC upload flow to mutate that tracker, and expose commands for snapshot/cleanup. In Vue, poll the backend snapshot from `App.vue`, show upload feedback globally, and refresh the file list when uploads finish.

**Tech Stack:** Rust, Tauri 2, Vue 3, TypeScript

---

### Task 1: Add upload tracker state
- [ ] Write failing Rust tests for enqueue, complete, and clear flows.
- [ ] Run `cargo test` for the new module and confirm failure.
- [ ] Implement `UploadTracker` with stable IDs and snapshot serialization.
- [ ] Re-run targeted Rust tests and confirm pass.

### Task 2: Route IPC uploads through tracker
- [ ] Add tracker to `AppState` and Tauri commands for snapshot/clear.
- [ ] Update pipe upload handling to enqueue before window show and mark results by ID.
- [ ] Harden `show_window()` with unminimize/show/focus sequence.
- [ ] Run targeted Rust tests again.

### Task 3: Replace frontend event dependency
- [ ] Remove upload event listeners from `App.vue`.
- [ ] Poll `get_uploads` from `App.vue` and manage clear timers.
- [ ] Move upload status UI to a global component and trigger file refresh on completion.
- [ ] Run `npm run build` to catch TS/Vue errors.

### Task 4: Verify end-to-end behavior
- [ ] Run `cargo test` in `src-tauri`.
- [ ] Run `npm run build` at repo root.
- [ ] Summarize manual right-click validation still needed in Windows shell.

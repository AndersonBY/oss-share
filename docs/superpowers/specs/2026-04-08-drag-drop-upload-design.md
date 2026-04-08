# Drag-and-Drop Upload Design

**Date:** 2026-04-08

## Goal

Allow users to drag one or more files onto any visible area of the OSS Share window and start uploading immediately.

## Scope

- Support dragging one or more files onto the main window.
- Start upload immediately on drop.
- Reuse the existing backend upload queue and frontend upload status UI.
- Show a full-window drop overlay while files are hovering.
- Reject folders with a clear message.

## Non-Goals

- Folder uploads.
- Recursive directory traversal.
- Per-byte percentage progress.

## Recommended Approach

Use Tauri window-level file drag/drop events instead of plain DOM drag events. This keeps the entire native window as a valid drop target and avoids browser-only drag limitations.

## Architecture

### Unified upload entrypoint

Introduce a single backend command that accepts a list of file paths and enqueues them into the same Rust upload tracker used by shell-extension uploads. Both right-click uploads and drag-and-drop uploads should call the same queueing path.

### Frontend drag controller

`App.vue` should subscribe to window-level drag/drop events. It owns three pieces of UI state:

- whether a drag is currently hovering the window
- which drop error message to show, if any
- the current tab switch to `files` when a drop succeeds

### Overlay UX

When files enter the window, show a full-window overlay with these messages:

- primary: `松手即可上传到 OSS`
- secondary: `支持多个文件，暂不支持文件夹`

Hide the overlay when the drag leaves or after the drop is handled.

### Validation rules

On drop:

1. Separate file paths from folder paths.
2. If at least one file exists, upload valid files immediately.
3. If any folder exists, show a visible error message that folders are not supported.
4. If nothing valid exists, do not enqueue uploads.

## Data Flow

1. User drags files into the OSS Share window.
2. Frontend receives Tauri drag event and shows overlay.
3. User drops files.
4. Frontend validates dropped paths and invokes backend enqueue command.
5. Backend appends items into `UploadTracker` and spawns upload work.
6. Existing polling refreshes upload banner and file list.

## Error Handling

- Folder dropped: show `暂不支持文件夹上传`.
- Invalid path or backend invoke failure: show `拖拽上传失败，请重试` plus console error.
- Mixed valid and invalid items: upload valid files and still show a folder warning.

## Files to Change

- `src/App.vue`
- `src/components/UploadStatusBanner.vue`
- `src/views/Files.vue` if a drop error banner belongs there
- `src-tauri/src/commands.rs`
- `src-tauri/src/ipc.rs`
- `src-tauri/src/main.rs`
- optional new helper in `src-tauri/src/upload_state.rs`

## Testing Strategy

- Add Rust tests for the shared upload enqueue path.
- Run `cargo test`.
- Run `npm run build`.
- Manual validation:
  - drag one file onto window
  - drag multiple files onto window
  - drag a folder onto window
  - verify queue appears and uploads complete

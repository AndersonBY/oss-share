# Drag-and-Drop Upload Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users drag one or more files onto any visible area of the OSS Share window and start uploading immediately.

**Architecture:** Add a shared Rust enqueue path that both shell-extension uploads and drag-and-drop uploads use, then subscribe to Tauri window drag events in `App.vue` to drive a full-window drop overlay and invoke the shared upload command. Reuse the existing upload tracker polling so dropped files appear in the same banner and refresh the file list when complete.

**Tech Stack:** Rust, Tauri 2, Vue 3, TypeScript

---

### Task 1: Shared backend enqueue path

**Files:**
- Modify: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/upload_state.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn begin_batch_returns_distinct_ids_for_multiple_files() {
    let tracker = UploadTracker::default();
    let queued = tracker.begin_batch(&[
        String::from(r"C:\tmp\a.txt"),
        String::from(r"C:\tmp\b.txt"),
    ]);

    assert_eq!(queued.len(), 2);
    assert_ne!(queued[0].id, queued[1].id);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test upload_state -- --nocapture`
Expected: FAIL before code is added.

- [ ] **Step 3: Write minimal implementation**

Create a shared Rust function that:

```rust
pub fn enqueue_uploads(app_handle: &AppHandle, files: Vec<String>) {
    let queued_uploads = {
        let state = app_handle.state::<AppState>();
        state.uploads.begin_batch(&files)
    };

    tray::show_window(app_handle, "files");
    spawn_upload_task(app_handle.clone(), queued_uploads);
}
```

And add a Tauri command that calls it:

```rust
#[tauri::command]
pub fn enqueue_uploads(stateful_app_handle: AppHandle, files: Vec<String>) -> Result<(), String> {
    ipc::enqueue_uploads(&stateful_app_handle, files);
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test upload_state -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/commands.rs src-tauri/src/upload_state.rs
git commit -m "feat: share upload enqueue path"
```

### Task 2: Drop validation contract

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/upload_state.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn clear_finished_does_not_remove_uploading_items() {
    let tracker = UploadTracker::default();
    let queued = tracker.begin_batch(&[String::from(r"C:\tmp\file.txt")]);
    tracker.clear_finished(&[queued[0].id]);
    assert_eq!(tracker.snapshot().len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test upload_state -- --nocapture`
Expected: FAIL before regression support exists.

- [ ] **Step 3: Write minimal implementation**

Return a structured command response for drag-and-drop uploads:

```rust
#[derive(Serialize)]
pub struct EnqueueUploadsResult {
    pub accepted_files: Vec<String>,
    pub rejected_directories: Vec<String>,
}
```

Validate dropped paths with `std::fs::metadata`, enqueue only files, and return rejected directories so the frontend can message them.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test upload_state -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/upload_state.rs
git commit -m "feat: validate dropped upload paths"
```

### Task 3: Whole-window drag overlay

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/DropOverlay.vue`
- Test: `src/App.vue`

- [ ] **Step 1: Write the failing UI expectation**

Document the desired state in code comments or a temporary assertion target:

```ts
const isDragActive = ref(false)
const dropError = ref("")
```

The overlay must appear on drag enter and disappear on drag leave/drop.

- [ ] **Step 2: Run build to verify current code lacks support**

Run: `npm run build`
Expected: current build passes but there is no drag overlay behavior yet.

- [ ] **Step 3: Write minimal implementation**

Subscribe to Tauri drag/drop events from `App.vue` and render:

```vue
<DropOverlay
  v-if="isDragActive"
  title="松手即可上传到 OSS"
  subtitle="支持多个文件，暂不支持文件夹"
/>
```

Track `isDragActive`, set `currentTab.value = "files"` on successful drop, and set `dropError` when directories are rejected.

- [ ] **Step 4: Run build to verify it passes**

Run: `npm run build`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/App.vue src/components/DropOverlay.vue
git commit -m "feat: add whole-window drag upload overlay"
```

### Task 4: Verify integrated behavior

**Files:**
- Modify: `src/views/Files.vue`
- Test: `src-tauri/src/main.rs`

- [ ] **Step 1: Wire file refresh and messaging**

Make sure dropped uploads reuse the existing refresh token path and show any folder warning near the top-level app UI.

- [ ] **Step 2: Run backend verification**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 3: Run frontend verification**

Run: `npm run build`
Expected: PASS.

- [ ] **Step 4: Manual validation checklist**

Run these checks manually:

```text
1. Drag one file into the window -> overlay shows -> upload starts immediately.
2. Drag multiple files -> all appear in upload banner.
3. Drag a folder -> no upload starts, folder warning appears.
4. Drag mixed files + folder -> files upload, folder warning appears.
```

- [ ] **Step 5: Commit**

```bash
git add src/App.vue src/views/Files.vue src-tauri/src/commands.rs src-tauri/src/ipc.rs
git commit -m "feat: support drag and drop uploads"
```

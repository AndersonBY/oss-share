<template>
  <div class="app">
    <div class="tabs">
      <button :class="{ active: currentTab === 'settings' }" @click="currentTab = 'settings'">设置</button>
      <button :class="{ active: currentTab === 'files' }" @click="currentTab = 'files'">文件管理</button>
    </div>
    <div v-if="dropError" class="drop-error">{{ dropError }}</div>
    <UploadStatusBanner :uploads="uploads" />
    <div v-show="currentTab === 'settings'">
      <Settings />
    </div>
    <div v-show="currentTab === 'files'">
      <Files :refresh-token="filesRefreshToken" />
    </div>
    <DropOverlay
      v-if="isDragActive"
      title="松手即可上传到 OSS"
      subtitle="支持多个文件，暂不支持文件夹"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Settings from "./views/Settings.vue";
import Files from "./views/Files.vue";
import UploadStatusBanner from "./components/UploadStatusBanner.vue";
import DropOverlay from "./components/DropOverlay.vue";
import type { EnqueueUploadsResult, UploadItem } from "./types/upload";

const currentTab = ref<"files" | "settings">("files");
const uploads = ref<UploadItem[]>([]);
const filesRefreshToken = ref(0);
const isDragActive = ref(false);
const dropError = ref("");

let pollTimer: number | undefined;
let clearTimer: number | undefined;
let clearFingerprint = "";
let lastSettledRefreshFingerprint = "";
let dropErrorTimer: number | undefined;
let unlistenDragDrop: (() => void) | undefined;

async function syncUploads() {
  try {
    uploads.value = await invoke<UploadItem[]>("get_uploads");
  } catch (error) {
    console.error("Failed to load uploads:", error);
  }
}

function stopPolling() {
  if (pollTimer !== undefined) {
    window.clearInterval(pollTimer);
    pollTimer = undefined;
  }
}

function stopClearTimer() {
  if (clearTimer !== undefined) {
    window.clearTimeout(clearTimer);
    clearTimer = undefined;
  }
  clearFingerprint = "";
}

function stopDropErrorTimer() {
  if (dropErrorTimer !== undefined) {
    window.clearTimeout(dropErrorTimer);
    dropErrorTimer = undefined;
  }
}

function setDropError(message: string) {
  dropError.value = message;
  stopDropErrorTimer();
  dropErrorTimer = window.setTimeout(() => {
    dropError.value = "";
    dropErrorTimer = undefined;
  }, 6000);
}

function handleWindowFocus() {
  void syncUploads();
}

async function handleDroppedPaths(paths: string[]) {
  try {
    const result = await invoke<EnqueueUploadsResult>("enqueue_uploads", { files: paths });

    if (result.accepted_files.length > 0) {
      currentTab.value = "files";
      await syncUploads();
    }

    if (result.rejected_directories.length > 0) {
      setDropError("暂不支持文件夹上传");
    } else if (result.accepted_files.length === 0) {
      setDropError("拖拽上传失败，请重试");
    }
  } catch (error) {
    console.error("Failed to upload dropped files:", error);
    setDropError("拖拽上传失败，请重试");
  }
}

watch(uploads, (list) => {
  const hasUploadingItems = list.some((upload) => upload.status === "uploading");
  const settledRefreshFingerprint = !hasUploadingItems && list.length > 0
    ? list.map((upload) => `${upload.id}:${upload.status}`).join("|")
    : "";

  if (
    settledRefreshFingerprint &&
    settledRefreshFingerprint !== lastSettledRefreshFingerprint
  ) {
    filesRefreshToken.value += 1;
  }
  lastSettledRefreshFingerprint = settledRefreshFingerprint;

  const completedIds = list
    .filter((upload) => upload.status !== "uploading")
    .map((upload) => upload.id)
    .sort((left, right) => left - right);
  const nextFingerprint = completedIds.join(",");

  if (!hasUploadingItems && completedIds.length > 0) {
    if (clearFingerprint !== nextFingerprint) {
      stopClearTimer();
      clearFingerprint = nextFingerprint;
      clearTimer = window.setTimeout(async () => {
        try {
          await invoke("clear_uploads", { ids: completedIds });
          await syncUploads();
        } catch (error) {
          console.error("Failed to clear uploads:", error);
        } finally {
          clearTimer = undefined;
          if (clearFingerprint === nextFingerprint) {
            clearFingerprint = "";
          }
        }
      }, 5000);
    }
  } else {
    stopClearTimer();
  }
});

// Theme
const theme = ref<"light" | "dark">(
  (localStorage.getItem("oss-share-theme") as "light" | "dark") || "light"
);

function applyTheme(t: string) {
  document.documentElement.setAttribute("data-theme", t);
  localStorage.setItem("oss-share-theme", t);
}

onMounted(() => applyTheme(theme.value));
watch(theme, (t) => applyTheme(t));

onMounted(() => {
  void syncUploads();
  pollTimer = window.setInterval(() => {
    void syncUploads();
  }, 1000);
  window.addEventListener("focus", handleWindowFocus);
  void getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === "enter" || event.payload.type === "over") {
      isDragActive.value = true;
      return;
    }

    if (event.payload.type === "leave") {
      isDragActive.value = false;
      return;
    }

    isDragActive.value = false;
    void handleDroppedPaths(event.payload.paths);
  }).then((unlisten) => {
    unlistenDragDrop = unlisten;
  });
});

onBeforeUnmount(() => {
  stopPolling();
  stopClearTimer();
  stopDropErrorTimer();
  window.removeEventListener("focus", handleWindowFocus);
  if (unlistenDragDrop) {
    unlistenDragDrop();
    unlistenDragDrop = undefined;
  }
});

// Expose for tray navigation and theme toggle
(window as any).__navigateTo = (tab: string) => {
  if (tab === "files" || tab === "settings") {
    currentTab.value = tab;
  }
};
(window as any).__setTheme = (t: "light" | "dark") => {
  theme.value = t;
};
(window as any).__getTheme = () => theme.value;
</script>

<style>
:root {
  --bg-primary: #f5f5f7;
  --bg-secondary: #ffffff;
  --bg-tertiary: #e8e8ed;
  --bg-input: #ffffff;
  --border: #d2d2d7;
  --text-primary: #1d1d1f;
  --text-secondary: #6e6e73;
  --text-muted: #aeaeb2;
  --accent: #4c8bf5;
  --accent-hover: #3a7be0;
  --accent-bg: #e8f0fe;
  --success: #34c759;
  --error: #ff3b30;
  --tab-active-bg: #4c8bf5;
  --tab-active-text: #ffffff;
  --tab-text: #6e6e73;
  --row-hover: #f0f0f5;
}

[data-theme="dark"] {
  --bg-primary: #0f0f1a;
  --bg-secondary: #1a1a2e;
  --bg-tertiary: #2a2a3e;
  --bg-input: #1a1a2e;
  --border: #3a3a4e;
  --text-primary: #e0e0e0;
  --text-secondary: #aaaaaa;
  --text-muted: #666666;
  --accent: #4c8bf5;
  --accent-hover: #6ca0ff;
  --accent-bg: #1a2a4a;
  --success: #4caf50;
  --error: #ff6b6b;
  --tab-active-bg: #3a3a5c;
  --tab-active-text: #ffffff;
  --tab-text: #888888;
  --row-hover: #222238;
}

body {
  margin: 0;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: 'Segoe UI', sans-serif;
}
.app { min-height: 100vh; }
.drop-error {
  margin: 16px 20px 0;
  border: 1px solid var(--error);
  border-radius: 8px;
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
  padding: 10px 14px;
  font-size: 13px;
}
.tabs { display: flex; border-bottom: 2px solid var(--border); padding: 0; }
.tabs button {
  padding: 10px 24px; background: transparent; border: none;
  color: var(--tab-text); cursor: pointer; font-size: 14px;
}
.tabs button.active {
  color: var(--tab-active-text);
  background: var(--tab-active-bg);
  border-radius: 6px 6px 0 0;
}
</style>

<template>
  <div class="files">
    <div v-if="listError" class="list-error">{{ listError }}</div>
    <div class="toolbar">
      <span class="file-count">{{ filteredFiles.length }} 个文件</span>
      <input v-model="search" type="text" placeholder="搜索文件名..." class="search-input" />
    </div>
    <div class="file-list">
      <div class="file-header">
        <span class="sortable" @click="toggleSort('key')">
          文件名 {{ sortArrow('key') }}
        </span>
        <span class="sortable" @click="toggleSort('size')">
          大小 {{ sortArrow('size') }}
        </span>
        <span class="sortable" @click="toggleSort('last_modified')">
          上传时间 {{ sortArrow('last_modified') }}
        </span>
        <span>操作</span>
      </div>
      <div v-if="loading" class="loading">加载中...</div>
      <div v-else-if="listError && filteredFiles.length === 0" class="empty">刷新失败，请稍后重试</div>
      <div v-else-if="filteredFiles.length === 0" class="empty">暂无文件</div>
      <FileRow v-for="file in filteredFiles" :key="file.key" :file="file"
        :copied="copiedKey === file.key"
        :deleting="deletingKey === file.key"
        @copy-link="copyLink" @delete="deleteFile" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import FileRow from "../components/FileRow.vue";

interface OssFile {
  key: string;
  size: number;
  last_modified: string;
}

const props = defineProps<{
  refreshToken: number;
}>();

const files = ref<OssFile[]>([]);
const search = ref("");
const loading = ref(true);
const copiedKey = ref("");
const deletingKey = ref("");
const listError = ref("");

watch(() => props.refreshToken, () => {
  void loadFiles();
});

// Sorting
const sortBy = ref<"key" | "size" | "last_modified">("last_modified");
const sortOrder = ref<"asc" | "desc">("desc");

function toggleSort(field: "key" | "size" | "last_modified") {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortBy.value = field;
    sortOrder.value = field === "key" ? "asc" : "desc";
  }
}

function sortArrow(field: string): string {
  if (sortBy.value !== field) return "";
  return sortOrder.value === "asc" ? "\u25B2" : "\u25BC";
}

const filteredFiles = computed(() => {
  let result = files.value;
  if (search.value) {
    const q = search.value.toLowerCase();
    result = result.filter((f) => f.key.toLowerCase().includes(q));
  }
  return [...result].sort((a, b) => {
    const field = sortBy.value;
    let cmp = 0;
    if (field === "size") {
      cmp = a.size - b.size;
    } else {
      cmp = String(a[field]).localeCompare(String(b[field]));
    }
    return sortOrder.value === "asc" ? cmp : -cmp;
  });
});

onMounted(() => {
  void loadFiles();
});

async function loadFiles() {
  loading.value = true;
  try {
    files.value = await invoke("list_files");
    listError.value = "";
  } catch (e) {
    console.error("Failed to list files:", e);
    listError.value = "刷新失败，请稍后重试";
  } finally {
    loading.value = false;
  }
}

async function copyLink(key: string) {
  try {
    const url: string = await invoke("get_share_link", { objectKey: key });
    await writeText(url);
    copiedKey.value = key;
    setTimeout(() => { copiedKey.value = ""; }, 2000);
  } catch (e) {
    console.error("Failed to get share link:", e);
  }
}

async function deleteFile(key: string) {
  if (deletingKey.value) {
    return;
  }

  deletingKey.value = key;
  try {
    await invoke("delete_file", { objectKey: key });
    files.value = files.value.filter((f) => f.key !== key);
    listError.value = "";
  } catch (e) {
    console.error("Failed to delete file:", e);
    listError.value = "删除失败，请稍后重试";
  } finally {
    deletingKey.value = "";
  }
}
</script>

<style scoped>
.files { padding: 20px; }
.list-error {
  margin-bottom: 12px;
  border: 1px solid var(--error);
  border-radius: 8px;
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
  padding: 10px 14px;
  font-size: 13px;
}

.toolbar {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;
}
.file-count { color: var(--text-secondary); font-size: 13px; }
.search-input {
  background: var(--bg-input); border: 1px solid var(--border); border-radius: 4px;
  padding: 4px 10px; color: var(--text-primary); font-size: 12px; width: 180px;
}
.file-list { border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
.file-header {
  display: grid; grid-template-columns: 1fr 100px 160px 120px;
  padding: 8px 12px; background: var(--bg-secondary); color: var(--text-secondary); font-size: 12px;
  border-bottom: 1px solid var(--border);
}
.file-header .sortable {
  cursor: pointer; user-select: none;
}
.file-header .sortable:hover { color: var(--accent); }
.loading, .empty { padding: 40px; text-align: center; color: var(--text-muted); }
</style>

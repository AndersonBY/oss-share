<template>
  <div class="file-row">
    <span class="file-name">{{ file.key.split('/').pop() }}</span>
    <span class="file-size">{{ formatSize(file.size) }}</span>
    <span class="file-date">{{ formatDate(file.last_modified) }}</span>
    <span class="file-actions">
      <button @click="$emit('copy-link', file.key)" :class="{ copied }" title="复制链接">
        {{ copied ? '已复制' : '复制链接' }}
      </button>
      <button
        class="btn-delete"
        :class="{ loading: deleting }"
        :disabled="deleting"
        @click="$emit('delete', file.key)"
        title="删除"
      >
        {{ deleting ? '删除中...' : '删除' }}
      </button>
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  file: { key: string; size: number; last_modified: string };
  copied: boolean;
  deleting: boolean;
}>();

defineEmits<{
  (e: "copy-link", key: string): void;
  (e: "delete", key: string): void;
}>();

function formatDate(isoStr: string): string {
  try {
    const d = new Date(isoStr);
    if (isNaN(d.getTime())) return isoStr;
    return d.toLocaleString("zh-CN", {
      year: "numeric", month: "2-digit", day: "2-digit",
      hour: "2-digit", minute: "2-digit",
    });
  } catch {
    return isoStr;
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
</script>

<style scoped>
.file-row {
  display: grid;
  grid-template-columns: 1fr 100px 160px 120px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  align-items: center;
}
.file-row:hover { background: var(--row-hover); }
.file-name { color: var(--text-primary); font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-size, .file-date { color: var(--text-secondary); font-size: 12px; }
.file-actions { display: flex; gap: 6px; }
.file-actions button {
  padding: 3px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;
  border: 1px solid var(--border); background: var(--bg-secondary); color: var(--text-primary);
}
.file-actions button:disabled {
  cursor: wait;
  opacity: 0.75;
}
.file-actions button:hover { background: var(--accent); color: #fff; border-color: var(--accent); }
.file-actions button.copied { background: var(--success); color: #fff; border-color: var(--success); }
.file-actions .btn-delete:hover { background: var(--error); color: #fff; border-color: var(--error); }
.file-actions .btn-delete.loading,
.file-actions .btn-delete.loading:hover {
  background: color-mix(in srgb, var(--error) 75%, var(--bg-secondary));
  color: #fff;
  border-color: var(--error);
}
</style>

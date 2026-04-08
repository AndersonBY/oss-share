<template>
  <div v-if="uploads.length" class="upload-area">
    <div v-for="upload in uploads" :key="upload.id" class="upload-item">
      <span class="upload-name">{{ upload.file_name }}</span>
      <span v-if="upload.status === 'uploading'" class="upload-status spinning">上传中...</span>
      <span v-else-if="upload.status === 'done'" class="upload-status done">已完成 - 链接已复制</span>
      <span v-else class="upload-status error">{{ upload.message || '失败' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { UploadItem } from "../types/upload";

defineProps<{
  uploads: UploadItem[];
}>();
</script>

<style scoped>
.upload-area {
  background: var(--accent-bg);
  border: 1px solid var(--accent);
  border-radius: 6px;
  padding: 12px;
  margin: 16px 20px 0;
}

.upload-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 6px 0;
}

.upload-name {
  color: var(--text-primary);
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.upload-status {
  flex-shrink: 0;
  font-size: 12px;
}

.upload-status.spinning { color: var(--accent); }
.upload-status.done { color: var(--success); }
.upload-status.error { color: var(--error); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.spinning { animation: pulse 1.5s infinite; }
</style>

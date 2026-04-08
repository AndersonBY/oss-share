<template>
  <div class="drop-overlay">
    <div class="drop-card">
      <div class="drop-title">{{ title }}</div>
      <div class="drop-subtitle">{{ subtitle }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  subtitle: string;
}>();
</script>

<style scoped>
.drop-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(76, 139, 245, 0.16);
  backdrop-filter: blur(4px);
  pointer-events: none;
}

.drop-card {
  min-width: min(420px, 100%);
  border: 2px dashed var(--accent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--bg-secondary) 88%, transparent);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.15);
  padding: 28px 32px;
  text-align: center;
}

.drop-title {
  color: var(--text-primary);
  font-size: 24px;
  font-weight: 700;
}

.drop-subtitle {
  margin-top: 10px;
  color: var(--text-secondary);
  font-size: 14px;
}
</style>

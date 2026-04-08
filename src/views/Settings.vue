<template>
  <div class="settings">
    <div class="form-group">
      <label>Access Key ID</label>
      <input v-model="form.accessKeyId" type="text" placeholder="LTAI5t..." />
    </div>
    <div class="form-group">
      <label>Access Key Secret</label>
      <input v-model="form.accessKeySecret" type="password" placeholder="输入 Access Key Secret" />
    </div>
    <div class="form-group">
      <label>Region</label>
      <input v-model="form.region" type="text" placeholder="oss-cn-hangzhou" />
    </div>
    <div class="form-group">
      <label>Bucket</label>
      <input v-model="form.bucket" type="text" placeholder="my-share-bucket" />
    </div>
    <div class="form-group">
      <label>上传路径前缀</label>
      <input v-model="form.prefix" type="text" placeholder="share/" />
    </div>
    <div class="form-group">
      <label>链接有效期</label>
      <div class="expire-options">
        <button v-for="opt in expireOptions" :key="opt.value"
          :class="{ active: form.expireSeconds === opt.value }"
          @click="form.expireSeconds = opt.value">
          {{ opt.label }}
        </button>
      </div>
    </div>
    <div class="actions">
      <button class="btn-secondary" @click="testConnection" :disabled="testing">
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button class="btn-primary" @click="save" :disabled="saving">
        {{ saving ? '保存中...' : '保存' }}
      </button>
    </div>
    <p v-if="message" :class="['message', messageType]">{{ message }}</p>

    <div class="divider"></div>
    <div class="form-group">
      <label>外观</label>
      <div class="theme-toggle">
        <button :class="{ active: currentTheme === 'light' }" @click="setTheme('light')">浅色</button>
        <button :class="{ active: currentTheme === 'dark' }" @click="setTheme('dark')">深色</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const form = ref({
  accessKeyId: "",
  accessKeySecret: "",
  region: "oss-cn-hangzhou",
  bucket: "",
  prefix: "share/",
  expireSeconds: 604800,
});

const expireOptions = [
  { label: "24h", value: 86400 },
  { label: "7d", value: 604800 },
  { label: "30d", value: 2592000 },
];

const testing = ref(false);
const saving = ref(false);
const message = ref("");
const messageType = ref<"success" | "error">("success");

const currentTheme = ref((window as any).__getTheme?.() || "light");

function setTheme(t: "light" | "dark") {
  currentTheme.value = t;
  (window as any).__setTheme?.(t);
}

onMounted(async () => {
  try {
    const config: any = await invoke("get_config");
    form.value.accessKeyId = config.credentials.access_key_id;
    form.value.accessKeySecret = config.credentials.access_key_secret;
    form.value.region = config.oss.region;
    form.value.bucket = config.oss.bucket;
    form.value.prefix = config.oss.prefix;
    form.value.expireSeconds = config.sharing.expire_seconds;
  } catch (e) {
    message.value = `加载配置失败: ${e}`;
    messageType.value = "error";
  }
});

async function save() {
  saving.value = true;
  message.value = "";
  try {
    await invoke("save_settings", {
      accessKeyId: form.value.accessKeyId,
      accessKeySecret: form.value.accessKeySecret,
      region: form.value.region,
      bucket: form.value.bucket,
      prefix: form.value.prefix,
      expireSeconds: form.value.expireSeconds,
    });
    message.value = "保存成功";
    messageType.value = "success";
  } catch (e) {
    message.value = `保存失败: ${e}`;
    messageType.value = "error";
  } finally {
    saving.value = false;
  }
}

async function testConnection() {
  testing.value = true;
  message.value = "";
  try {
    await invoke("test_connection");
    message.value = "连接成功";
    messageType.value = "success";
  } catch (e) {
    message.value = `连接失败: ${e}`;
    messageType.value = "error";
  } finally {
    testing.value = false;
  }
}
</script>

<style scoped>
.settings { padding: 20px; }
.form-group {
  display: grid;
  grid-template-columns: 120px 1fr;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}
.form-group label { text-align: right; color: var(--text-secondary); font-size: 13px; }
.form-group input {
  background: var(--bg-input); border: 1px solid var(--border); border-radius: 4px;
  padding: 6px 10px; color: var(--text-primary); font-size: 13px;
}
.expire-options, .theme-toggle { display: flex; gap: 8px; }
.expire-options button, .theme-toggle button {
  padding: 6px 14px; background: var(--bg-secondary); border: 1px solid var(--border);
  border-radius: 4px; color: var(--text-secondary); cursor: pointer;
}
.expire-options button.active, .theme-toggle button.active {
  background: var(--accent); border-color: var(--accent); color: #fff;
}
.actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px; }
.btn-primary {
  padding: 6px 20px; background: var(--accent); border: none;
  border-radius: 4px; color: #fff; cursor: pointer;
}
.btn-primary:hover { background: var(--accent-hover); }
.btn-secondary {
  padding: 6px 20px; background: var(--bg-tertiary); border: 1px solid var(--border);
  border-radius: 4px; color: var(--text-primary); cursor: pointer;
}
.message { margin-top: 12px; font-size: 13px; }
.message.success { color: var(--success); }
.message.error { color: var(--error); }
.divider { border-top: 1px solid var(--border); margin: 20px 0; }
</style>

<!-- src/components/settings/SettingsPopup.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { check } from '@tauri-apps/plugin-updater'
import UpdateDialog from '@/components/common/UpdateDialog.vue'
import AboutDialog from '@/components/common/AboutDialog.vue'
import { RefreshCw, Info } from 'lucide-vue-next'

const { t } = useI18n()
const message = useMessage()

const showUpdateDialog = ref(false)
const showAboutDialog = ref(false)
const checking = ref(false)

const handleCheckUpdate = async () => {
  checking.value = true
  try {
    const update = await check()
    checking.value = false
    if (update) {
      showUpdateDialog.value = true
    } else {
      message.success(t('updater.noUpdate'))
    }
  } catch (e) {
    checking.value = false
    message.error(t('updater.checkFailed') + ': ' + String(e))
  }
}

const handleAbout = () => {
  showAboutDialog.value = true
}

const appVersion = '0.2.6'
</script>

<template>
  <div class="settings-popup">
    <div class="popup-header">{{ t('common.settings') }}</div>

    <div class="popup-list">
      <div class="popup-item" @click="handleCheckUpdate">
        <RefreshCw :size="14" :class="{ 'loading-spinner': checking }" />
        <span class="item-text">{{ t('updater.title') }}</span>
        <span class="item-version">v{{ appVersion }}</span>
      </div>

      <div class="popup-item" @click="handleAbout">
        <Info :size="14" />
        <span class="item-text">{{ t('settings.about') }}</span>
      </div>
    </div>
  </div>

  <UpdateDialog v-model:show="showUpdateDialog" />
  <AboutDialog v-model:show="showAboutDialog" />
</template>

<style scoped>
.settings-popup {
  width: 200px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.popup-header {
  padding: 8px 12px;
  font-size: 11px;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.popup-list {
  padding: 4px 0;
}

.popup-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background 0.15s ease;
  color: var(--text-primary);
}

.popup-item:hover {
  background: var(--bg-hover);
}

.item-text {
  flex: 1;
  font-size: 13px;
}

.item-version {
  font-size: 11px;
  color: var(--text-muted);
}

.loading-spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
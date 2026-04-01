<!-- src/components/settings/SettingsPopup.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import AboutDialog from '@/components/common/AboutDialog.vue'
import { Info } from 'lucide-vue-next'

const { t } = useI18n()

const showAboutDialog = ref(false)

const handleAbout = () => {
  showAboutDialog.value = true
}

const appVersion = '0.2.11'
</script>

<template>
  <div class="settings-popup">
    <div class="popup-header">{{ t('common.settings') }}</div>

    <div class="popup-list">
      <div class="popup-item" @click="handleAbout">
        <Info :size="14" />
        <span class="item-text">{{ t('settings.about') }}</span>
        <span class="item-version">v{{ appVersion }}</span>
      </div>
    </div>
  </div>

  <AboutDialog v-model:show="showAboutDialog" />
</template>

<style scoped>
.settings-popup {
  width: 200px;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-radius: var(--border-radius);
  box-shadow: var(--glass-shadow);
  overflow: hidden;
}

.popup-header {
  padding: 8px 12px;
  font-size: 11px;
  color: var(--text-muted);
  border-bottom: 1px solid var(--glass-border);
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
  background: var(--glass-bg-hover);
}

.item-text {
  flex: 1;
  font-size: 13px;
}

.item-version {
  font-size: 11px;
  color: var(--text-muted);
}

/* Light Mode */
:root[data-theme="light"] .settings-popup {
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(59, 130, 246, 0.2);
  box-shadow: 0 4px 16px rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .popup-header {
  color: #64748b;
  border-bottom: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .popup-item {
  color: #1e293b;
}

:root[data-theme="light"] .popup-item:hover {
  background: rgba(59, 130, 246, 0.1);
}

:root[data-theme="light"] .item-version {
  color: #64748b;
}
</style>
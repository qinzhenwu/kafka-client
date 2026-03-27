<script setup lang="ts">
import { watch } from 'vue'
import { useUpdater } from '@/composables/useUpdater'
import { useI18n } from 'vue-i18n'
import { Download, X, RefreshCw } from 'lucide-vue-next'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const { t } = useI18n()
const {
  checking,
  updateAvailable,
  downloading,
  updateInfo,
  error,
  checkForUpdate,
  downloadAndInstall
} = useUpdater()

// Check for update when dialog opens
watch(() => props.show, async (show) => {
  if (show && !updateAvailable.value && !checking.value) {
    await checkForUpdate()
  }
})

const handleClose = () => {
  emit('update:show', false)
}

const handleUpdate = async () => {
  await downloadAndInstall()
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleClose">
      <div class="modal-container" @click.stop>
        <!-- Header -->
        <div class="modal-header">
          <span class="header-title">
            <RefreshCw :size="18" :stroke-width="1.5" class="header-icon" />
            {{ t('updater.title', '软件更新') }}
          </span>
          <X :size="16" class="header-close" @click="handleClose" />
        </div>

        <!-- Content -->
        <div class="modal-content">
          <!-- Checking -->
          <div v-if="checking" class="checking-state">
            <RefreshCw :size="32" class="spinning" />
            <p>{{ t('updater.checking', '正在检查更新...') }}</p>
          </div>

          <!-- No Update -->
          <div v-else-if="!updateAvailable && !error" class="no-update-state">
            <p>{{ t('updater.noUpdate', '当前已是最新版本') }}</p>
            <p class="version-info">{{ t('updater.currentVersion', '当前版本') }}: v{{ updateInfo?.currentVersion }}</p>
          </div>

          <!-- Update Available -->
          <div v-else-if="updateAvailable && updateInfo" class="update-available">
            <p class="update-title">
              {{ t('updater.newVersionAvailable', '发现新版本') }}: v{{ updateInfo.version }}
            </p>
            <p class="version-info">{{ t('updater.currentVersion', '当前版本') }}: v{{ updateInfo.currentVersion }}</p>
            <div v-if="updateInfo.body" class="release-notes">
              <p class="notes-label">{{ t('updater.releaseNotes', '更新内容') }}:</p>
              <pre class="notes-content">{{ updateInfo.body }}</pre>
            </div>
          </div>

          <!-- Error -->
          <div v-else-if="error" class="error-state">
            <p>{{ t('updater.checkFailed', '检查更新失败') }}</p>
            <p class="error-msg">{{ error }}</p>
          </div>
        </div>

        <!-- Footer -->
        <div class="modal-footer">
          <span class="footer-hint">{{ t('updater.hint', '保持软件更新以获得最佳体验') }}</span>
          <div class="footer-actions">
            <button class="action-btn" @click="handleClose">{{ t('common.cancel') }}</button>
            <button
              v-if="updateAvailable"
              class="action-btn primary"
              :disabled="downloading"
              @click="handleUpdate"
            >
              <Download :size="14" :class="{ 'spinning': downloading }" />
              {{ downloading ? t('updater.downloading', '下载中...') : t('updater.updateNow', '立即更新') }}
            </button>
            <button
              v-else
              class="action-btn"
              :disabled="checking"
              @click="checkForUpdate"
            >
              <RefreshCw :size="14" :class="{ 'spinning': checking }" />
              {{ t('updater.checkAgain', '重新检查') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.modal-container {
  width: 420px;
  max-width: 95vw;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-icon {
  color: var(--accent);
}

.header-close {
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.header-close:hover {
  color: var(--text-primary);
}

.modal-content {
  padding: 24px 20px;
  min-height: 120px;
}

.checking-state,
.no-update-state,
.error-state {
  text-align: center;
  color: var(--text-secondary);
}

.checking-state p,
.no-update-state p,
.error-state p {
  margin: 12px 0 0;
}

.spinning {
  animation: spin 1s linear infinite;
  color: var(--accent);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.version-info {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 8px;
}

.update-available {
  color: var(--text-primary);
}

.update-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--success);
  margin-bottom: 4px;
}

.release-notes {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.notes-label {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.notes-content {
  background: var(--bg-tertiary);
  border-radius: 8px;
  padding: 12px;
  font-size: 13px;
  color: var(--text-secondary);
  max-height: 150px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
}

.error-state .error-msg {
  font-size: 12px;
  color: var(--error);
  background: var(--error-bg);
  padding: 8px 12px;
  border-radius: 6px;
  margin-top: 12px;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
}

.footer-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
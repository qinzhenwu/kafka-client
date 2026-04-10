<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useClusterStore, type ClusterItem } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import ClusterForm from '@/components/cluster/ClusterForm.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import { Server, Plus, X, Loader2 } from 'lucide-vue-next'

const { t } = useI18n()
const message = useMessage()
const clusterStore = useClusterStore()
const tabStore = useTabStore()

const emit = defineEmits<{
  close: []
}>()

const showForm = ref(false)
const editingCluster = ref<ClusterItem | null>(null)
const showDeleteConfirm = ref(false)
const pendingDeleteClusterId = ref<string | null>(null)
const connectingClusterId = ref<string | null>(null)

const handleConnect = async (clusterId: string) => {
  const cluster = clusterStore.clusters.find(c => c.id === clusterId)
  if (cluster) {
    connectingClusterId.value = clusterId
    try {
      const clientId = await clusterStore.connectCluster(cluster.config)
      // 添加集群 Tab
      tabStore.addClusterTab({
        clusterId: cluster.id,
        clusterName: cluster.name,
        clientId,
        connected: true
      })
      message.success(t('cluster.connectSuccess'))
    } catch (e: unknown) {
      message.error(t('cluster.connectFailed') + ': ' + String(e))
    } finally {
      connectingClusterId.value = null
    }
  }
}

const handleDisconnect = async (clientId: string) => {
  // 找到对应的 tab 并移除
  const tab = tabStore.clusterTabs.find(t => t.clientId === clientId)
  if (tab) {
    tabStore.removeClusterTab(tab.id)
  }
  // 断开连接
  await clusterStore.disconnectCluster(clientId)
}

const handleEdit = (clusterId: string) => {
  const cluster = clusterStore.clusters.find(c => c.id === clusterId)
  if (cluster) {
    editingCluster.value = cluster
    showForm.value = true
  }
}

const handleDelete = (clusterId: string) => {
  pendingDeleteClusterId.value = clusterId
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  if (pendingDeleteClusterId.value) {
    await clusterStore.deleteCluster(pendingDeleteClusterId.value)
    pendingDeleteClusterId.value = null
  }
}

const cancelDelete = () => {
  pendingDeleteClusterId.value = null
}

const handleAddCluster = () => {
  editingCluster.value = null
  showForm.value = true
}

const handleFormSuccess = () => {
  showForm.value = false
  editingCluster.value = null
}

const handleFormClose = (value: boolean) => {
  showForm.value = value
  if (!value) {
    editingCluster.value = null
  }
}

const clusterCount = computed(() => clusterStore.clusters.length)
</script>

<template>
  <div class="cluster-manager-overlay" @click="emit('close')">
    <div class="cluster-manager" @click.stop>
      <!-- Header -->
      <div class="manager-header">
        <span class="header-title">{{ t('cluster.title') }}</span>
        <X :size="18" class="header-close" @click="emit('close')" />
      </div>

      <!-- Content -->
      <div class="manager-content">
        <div
          v-for="cluster in clusterStore.clusters"
          :key="cluster.id"
          class="cluster-card"
          :class="{ connected: cluster.connected }"
        >
          <div class="card-header">
            <div class="card-status">
              <span class="status-dot" :class="cluster.connected ? 'connected' : 'disconnected'"></span>
              <span class="status-text">{{ cluster.connected ? t('cluster.connected') : t('cluster.disconnected') }}</span>
            </div>
            <div class="card-actions">
              <template v-if="cluster.connected">
                <button class="action-btn edit-btn" @click="handleEdit(cluster.id)">{{ t('common.edit') }}</button>
                <button class="action-btn disconnect-btn" @click="handleDisconnect(cluster.clientId!)">{{ t('cluster.disconnect') }}</button>
              </template>
              <template v-else>
                <button class="action-btn primary" :disabled="connectingClusterId === cluster.id" @click="handleConnect(cluster.id)">
                  <Loader2 v-if="connectingClusterId === cluster.id" :size="12" class="loading-spinner" />
                  <span v-if="connectingClusterId === cluster.id">{{ t('cluster.connecting') }}</span>
                  <span v-else>{{ t('cluster.connect') }}</span>
                </button>
                <button class="action-btn edit-btn" :disabled="connectingClusterId === cluster.id" @click="handleEdit(cluster.id)">{{ t('common.edit') }}</button>
                <button class="action-btn danger" :disabled="connectingClusterId === cluster.id" @click="handleDelete(cluster.id)">{{ t('common.delete') }}</button>
              </template>
            </div>
          </div>
          <div class="card-body">
            <Server :size="16" :stroke-width="1.5" class="cluster-icon" />
            <span class="cluster-name">{{ cluster.name }}</span>
          </div>
          <div class="card-footer">
            {{ cluster.config.bootstrap_servers }}
          </div>
        </div>

        <div v-if="clusterStore.clusters.length === 0" class="empty-state">
          <span>{{ t('cluster.noClusters') }}</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="manager-footer">
        <span class="footer-count">{{ t('cluster.clusterCount', { count: clusterCount }) }}</span>
        <button class="add-btn" @click="handleAddCluster">
          <Plus :size="14" :stroke-width="1.5" />
          <span>{{ t('cluster.addCluster') }}</span>
        </button>
      </div>

      <!-- Cluster Form Modal -->
      <Teleport to="body">
        <ClusterForm
          :show="showForm"
          :edit-cluster="editingCluster"
          @update:show="handleFormClose"
          @success="handleFormSuccess"
        />
      </Teleport>

      <!-- Delete Confirm Dialog -->
      <Teleport to="body">
        <ConfirmDialog
          :show="showDeleteConfirm"
          :title="t('cluster.confirmDelete')"
          :message="t('cluster.deleteConfirmText', { name: pendingDeleteClusterId ? clusterStore.clusters.find(c => c.id === pendingDeleteClusterId)?.name : '' })"
          :confirm-text="t('common.confirm')"
          :cancel-text="t('common.cancel')"
          @confirm="confirmDelete"
          @cancel="cancelDelete"
          @close="showDeleteConfirm = false"
        />
      </Teleport>
    </div>
  </div>
</template>

<style scoped>
.cluster-manager-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.cluster-manager {
  width: 480px;
  max-height: 80vh;
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--glass-border);
}

.header-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.header-close {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.header-close:hover {
  color: var(--text-primary);
}

.manager-content {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
}

.cluster-card {
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 14px;
  margin-bottom: 12px;
}

.cluster-card.connected {
  border-color: rgba(34, 197, 94, 0.5);
  box-shadow: 0 0 12px rgba(34, 197, 94, 0.15);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.card-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  box-shadow: 0 0 6px currentColor;
}

.status-dot.connected {
  background: #22c55e;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.status-dot.disconnected {
  background: #ef4444;
  box-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
}

.status-text {
  font-size: 11px;
  font-weight: 500;
  color: #f1f5f9;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
}

.card-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  padding: 5px 12px;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

/* 编辑按钮 - 蓝色 */
.action-btn.edit-btn {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
  color: #93c5fd;
}

.action-btn.edit-btn:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.5);
  color: #bfdbfe;
}

/* 断开连接按钮 - 橙色 */
.action-btn.disconnect-btn {
  background: rgba(249, 115, 22, 0.15);
  border-color: rgba(249, 115, 22, 0.3);
  color: #fdba74;
}

.action-btn.disconnect-btn:hover:not(:disabled) {
  background: rgba(249, 115, 22, 0.25);
  border-color: rgba(249, 115, 22, 0.5);
  color: #fed7aa;
}

/* 连接按钮 - 绿色 */
.action-btn.primary {
  background: rgba(34, 197, 94, 0.2);
  border-color: rgba(34, 197, 94, 0.4);
  color: #86efac;
}

.action-btn.primary:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(34, 197, 94, 0.6);
  color: #bbf7d0;
}

/* 删除按钮 - 红色 */
.action-btn.danger {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
  color: #fca5a5;
}

.action-btn.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.25);
  border-color: rgba(239, 68, 68, 0.5);
  color: #fecaca;
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

.card-body {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.cluster-icon {
  color: var(--text-primary);
  flex-shrink: 0;
}

.cluster-name {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
}

.card-footer {
  font-size: 12px;
  color: #cbd5e1;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.empty-state {
  padding: 32px;
  text-align: center;
  color: var(--text-muted);
}

.manager-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--glass-border);
}

.footer-count {
  font-size: 12px;
  color: var(--text-muted);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: rgba(34, 197, 94, 0.2);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(34, 197, 94, 0.4);
  border-radius: 6px;
  color: #86efac;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.add-btn:hover {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(34, 197, 94, 0.6);
  color: #bbf7d0;
}

/* Light Mode */
:root[data-theme="light"] .cluster-manager-overlay {
  background: rgba(0, 0, 0, 0.2);
}

:root[data-theme="light"] .cluster-manager {
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(0, 0, 0, 0.15);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

:root[data-theme="light"] .manager-header {
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .header-title {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .header-close {
  color: #64748b;
}

:root[data-theme="light"] .header-close:hover {
  color: #1e293b;
}

:root[data-theme="light"] .cluster-card {
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .cluster-card.connected {
  border-color: rgba(34, 197, 94, 0.4);
  box-shadow: 0 0 12px rgba(34, 197, 94, 0.1);
}

:root[data-theme="light"] .status-text {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .cluster-name {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .card-footer {
  color: #475569;
  text-shadow: none;
}

:root[data-theme="light"] .action-btn {
  background: rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.15);
  color: #475569;
  text-shadow: none;
}

:root[data-theme="light"] .action-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.1);
  color: #1e293b;
  border-color: rgba(0, 0, 0, 0.25);
}

:root[data-theme="light"] .action-btn.edit-btn {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.25);
  color: #3b82f6;
}

:root[data-theme="light"] .action-btn.edit-btn:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.4);
  color: #2563eb;
}

:root[data-theme="light"] .action-btn.disconnect-btn {
  background: rgba(249, 115, 22, 0.1);
  border-color: rgba(249, 115, 22, 0.25);
  color: #f97316;
}

:root[data-theme="light"] .action-btn.disconnect-btn:hover:not(:disabled) {
  background: rgba(249, 115, 22, 0.2);
  border-color: rgba(249, 115, 22, 0.4);
  color: #ea580c;
}

:root[data-theme="light"] .action-btn.primary {
  background: rgba(34, 197, 94, 0.15);
  border-color: rgba(34, 197, 94, 0.3);
  color: #16a34a;
}

:root[data-theme="light"] .action-btn.primary:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.25);
  border-color: rgba(34, 197, 94, 0.5);
  color: #15803d;
}

:root[data-theme="light"] .action-btn.danger {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.25);
  color: #ef4444;
}

:root[data-theme="light"] .action-btn.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.4);
  color: #dc2626;
}

:root[data-theme="light"] .manager-footer {
  border-top: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .footer-count {
  color: #64748b;
  text-shadow: none;
}

:root[data-theme="light"] .add-btn {
  background: rgba(34, 197, 94, 0.15);
  border-color: rgba(34, 197, 94, 0.3);
  color: #16a34a;
  text-shadow: none;
}

:root[data-theme="light"] .add-btn:hover {
  background: rgba(34, 197, 94, 0.25);
  border-color: rgba(34, 197, 94, 0.5);
  color: #15803d;
}

:root[data-theme="light"] .empty-state {
  color: #64748b;
}
</style>
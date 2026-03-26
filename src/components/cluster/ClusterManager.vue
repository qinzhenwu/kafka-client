<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useClusterStore, type ClusterItem } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import ClusterForm from '@/components/cluster/ClusterForm.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import { Server, Plus, X } from 'lucide-vue-next'

const { t } = useI18n()
const clusterStore = useClusterStore()
const tabStore = useTabStore()

const emit = defineEmits<{
  close: []
}>()

const showForm = ref(false)
const editingCluster = ref<ClusterItem | null>(null)
const showDeleteConfirm = ref(false)
const pendingDeleteClusterId = ref<string | null>(null)

const handleConnect = async (clusterId: string) => {
  const cluster = clusterStore.clusters.find(c => c.id === clusterId)
  if (cluster) {
    const clientId = await clusterStore.connectCluster(cluster.config)
    // 添加集群 Tab
    tabStore.addClusterTab({
      clusterId: cluster.id,
      clusterName: cluster.name,
      clientId,
      connected: true
    })
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
                <button class="action-btn" @click="handleEdit(cluster.id)">{{ t('common.edit') }}</button>
                <button class="action-btn" @click="handleDisconnect(cluster.clientId!)">{{ t('cluster.disconnect') }}</button>
              </template>
              <template v-else>
                <button class="action-btn primary" @click="handleConnect(cluster.id)">{{ t('cluster.connect') }}</button>
                <button class="action-btn" @click="handleEdit(cluster.id)">{{ t('common.edit') }}</button>
                <button class="action-btn danger" @click="handleDelete(cluster.id)">{{ t('common.delete') }}</button>
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
      <ClusterForm
        :show="showForm"
        :edit-cluster="editingCluster"
        @update:show="handleFormClose"
        @success="handleFormSuccess"
      />

      <!-- Delete Confirm Dialog -->
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
    </div>
  </div>
</template>

<style scoped>
.cluster-manager-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.cluster-manager {
  width: 480px;
  max-height: 80vh;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.header-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
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
  background: var(--bg-tertiary);
  border-radius: 8px;
  padding: 14px;
  margin-bottom: 12px;
  border: 1px solid transparent;
}

.cluster-card.connected {
  border-color: var(--success);
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
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot.connected {
  background: var(--success);
}

.status-dot.disconnected {
  background: var(--text-muted);
}

.status-text {
  font-size: 10px;
  color: var(--text-muted);
}

.card-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  padding: 4px 10px;
  background: var(--bg-secondary);
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.danger {
  color: var(--error);
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
  font-size: 15px;
  color: var(--text-primary);
}

.card-footer {
  font-size: 12px;
  color: var(--text-muted);
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
  border-top: 1px solid var(--border);
}

.footer-count {
  font-size: 12px;
  color: var(--text-muted);
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--accent);
  border: none;
  border-radius: 6px;
  color: white;
  font-size: 13px;
  cursor: pointer;
}

.add-btn:hover {
  background: var(--accent-hover);
}
</style>
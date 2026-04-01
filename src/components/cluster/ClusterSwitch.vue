<!-- src/components/cluster/ClusterSwitch.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useClusterStore } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import { useMessageStore } from '@/stores/message'
import { focusClusterWindow } from '@/utils/window'
import { Loader2, Link2Off } from 'lucide-vue-next'

const { t } = useI18n()
const message = useMessage()
const clusterStore = useClusterStore()
const tabStore = useTabStore()
const messageStore = useMessageStore()

const emit = defineEmits<{
  close: []
  'open-manager': []
}>()

const connectingClusterId = ref<string | null>(null)

// 打开时同步连接状态
onMounted(async () => {
  console.log('[ClusterSwitch] Mounted, syncing connection status')
  await clusterStore.syncConnectionStatus()
})

const sortedClusters = computed(() => {
  return [...clusterStore.clusters].sort((a, b) => {
    // Connected clusters first
    if (a.connected && !b.connected) return -1
    if (!a.connected && b.connected) return 1
    return 0
  })
})

const handleSwitchCluster = async (clusterId: string) => {
  // 如果正在连接其他集群，不允许操作
  if (connectingClusterId.value) return

  const cluster = clusterStore.clusters.find(c => c.id === clusterId)
  if (!cluster) {
    console.log('[ClusterSwitch] Cluster not found')
    return
  }

  console.log('[ClusterSwitch] handleSwitchCluster:', { clusterId, connected: cluster.connected, clientId: cluster.clientId })

  // 如果未连接，先连接
  if (!cluster.connected) {
    connectingClusterId.value = clusterId
    try {
      console.log('[ClusterSwitch] Connecting to cluster...')
      const clientId = await clusterStore.connectCluster(cluster.config)
      console.log('[ClusterSwitch] Connected, clientId:', clientId)
      // 添加集群 Tab
      tabStore.addClusterTab({
        clusterId: cluster.id,
        clusterName: cluster.name,
        clientId,
        connected: true
      })
      console.log('[ClusterSwitch] Tab added, activeClientId:', tabStore.activeClientId)
      message.success(t('cluster.connectSuccess'))
      emit('close')
    } catch (e) {
      console.error('Failed to connect cluster:', e)
      message.error(t('cluster.connectFailed') + ': ' + String(e))
    } finally {
      connectingClusterId.value = null
    }
  } else {
    // 已连接，检查是否在当前窗口的 Tab 中
    const clientId = cluster.clientId || cluster.id
    const existingTab = tabStore.clusterTabs.find(t => t.clientId === clientId)

    console.log('[ClusterSwitch] Already connected, existingTab in current window:', !!existingTab)

    if (existingTab) {
      // 在当前窗口，切换到该 Tab
      tabStore.switchClusterTab(existingTab.id)
      console.log('[ClusterSwitch] Switched to existing tab')
    } else {
      // 可能在其他窗口，尝试聚焦
      const result = await focusClusterWindow(clientId)
      console.log('[ClusterSwitch] focusClusterWindow result:', result)

      if (!result) {
        // 不在任何窗口（不应该发生，但作为容错），在当前窗口添加 Tab
        tabStore.addClusterTab({
          clusterId: cluster.id,
          clusterName: cluster.name,
          clientId,
          connected: true
        })
        console.log('[ClusterSwitch] Tab added to current window')
      }
      // 如果 result 是 'main-window' 或 'cluster-window'，说明成功聚焦了对应窗口
    }
    emit('close')
  }
}

const handleOpenManager = () => {
  emit('open-manager')
  emit('close')
}

const disconnectingClusterId = ref<string | null>(null)

const handleDisconnect = async (clusterId: string, event: Event) => {
  // 阻止事件冒泡，避免触发 handleSwitchCluster
  event.stopPropagation()

  const cluster = clusterStore.clusters.find(c => c.id === clusterId)
  if (!cluster || !cluster.connected || !cluster.clientId) return

  disconnectingClusterId.value = clusterId
  const clientId = cluster.clientId

  try {
    console.log('[ClusterSwitch] Disconnecting cluster:', clientId)

    // 1. 停止该集群的所有实时消费
    const clusterTab = tabStore.clusterTabs.find(t => t.clientId === clientId)
    if (clusterTab && messageStore.isStreaming && messageStore.streamTopic) {
      // 检查正在消费的 topic 是否属于该集群
      const hasMessageTab = clusterTab.contentTabs.some(t =>
        t.type === 'message' && t.params.topicName === messageStore.streamTopic
      )
      if (hasMessageTab) {
        console.log('[ClusterSwitch] Stopping realtime consume for topic:', messageStore.streamTopic)
        await messageStore.stopRealtimeConsume(clientId, messageStore.streamTopic)
      }
    }

    // 2. 断开集群连接
    await clusterStore.disconnectCluster(clientId)
    console.log('[ClusterSwitch] Cluster disconnected')

    // 3. 关闭对应的集群 Tab
    const tabId = `cluster-${clientId}`
    tabStore.removeClusterTab(tabId)
    console.log('[ClusterSwitch] Cluster tab removed')

    message.success(t('cluster.disconnectSuccess'))
  } catch (e) {
    console.error('[ClusterSwitch] Failed to disconnect:', e)
    message.error(t('cluster.disconnectFailed') + ': ' + String(e))
  } finally {
    disconnectingClusterId.value = null
  }
}
</script>

<template>
  <div class="cluster-switch">
    <div class="switch-header">{{ t('cluster.quickSwitch') }}</div>

    <div class="cluster-list">
      <div
        v-for="cluster in sortedClusters"
        :key="cluster.id"
        class="cluster-item"
        :class="{ active: cluster.clientId === clusterStore.activeClusterId, connecting: connectingClusterId === cluster.id }"
        @click="handleSwitchCluster(cluster.id)"
      >
        <div class="cluster-status">
          <Loader2 v-if="connectingClusterId === cluster.id" :size="10" class="loading-spinner" />
          <span v-else class="status-dot" :class="cluster.connected ? 'connected' : 'disconnected'"></span>
          <span class="status-text">
            <template v-if="connectingClusterId === cluster.id">{{ t('cluster.connecting') }}</template>
            <template v-else>{{ cluster.connected ? t('cluster.connected') : t('cluster.clickToConnect') }}</template>
          </span>
        </div>
        <div class="cluster-info">
          <span class="cluster-icon">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
            </svg>
          </span>
          <span class="cluster-name">{{ cluster.name }}</span>
          <Loader2 v-if="connectingClusterId === cluster.id" :size="14" class="loading-spinner-right" />
          <button
            v-if="cluster.connected && disconnectingClusterId !== cluster.id"
            class="disconnect-btn"
            :title="t('cluster.disconnect')"
            @click="handleDisconnect(cluster.id, $event)"
          >
            <Link2Off :size="12" />
          </button>
          <Loader2 v-if="disconnectingClusterId === cluster.id" :size="12" class="loading-spinner-right" />
        </div>
      </div>

      <div v-if="clusterStore.clusters.length === 0" class="empty-list">
        <span>{{ t('cluster.noClusters') }}</span>
      </div>
    </div>

    <div class="switch-footer" @click="handleOpenManager">
      <span class="footer-icon">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
        </svg>
      </span>
      <span class="footer-text">{{ t('cluster.manageClusters') }}</span>
    </div>
  </div>
</template>

<style scoped>
.cluster-switch {
  width: 200px;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-radius: var(--border-radius);
  box-shadow: var(--glass-shadow);
  overflow: hidden;
}

.switch-header {
  padding: 8px 12px;
  font-size: 11px;
  color: var(--text-muted);
  border-bottom: 1px solid var(--glass-border);
}

.cluster-list {
  max-height: 240px;
  overflow-y: auto;
}

.cluster-item {
  padding: 10px 12px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.cluster-item:hover {
  background: var(--glass-bg-hover);
}

.cluster-item.active {
  background: var(--glass-bg-active);
  border-left: 2px solid var(--accent);
}

.cluster-item.connecting {
  cursor: wait;
}

.cluster-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
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

.cluster-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.disconnect-btn {
  margin-left: auto;
  padding: 4px;
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: #ef4444;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.disconnect-btn:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #dc2626;
}

.loading-spinner {
  color: var(--accent);
  animation: spin 1s linear infinite;
}

.loading-spinner-right {
  color: var(--accent);
  animation: spin 1s linear infinite;
  margin-left: auto;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.cluster-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.cluster-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-list {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.switch-footer {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  border-top: 1px solid var(--glass-border);
  cursor: pointer;
  color: var(--accent);
  transition: background 0.15s ease;
}

.switch-footer:hover {
  background: var(--glass-bg-hover);
}

.footer-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.footer-text {
  font-size: 13px;
}

/* Light Mode */
:root[data-theme="light"] .cluster-switch {
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid rgba(59, 130, 246, 0.2);
  box-shadow: 0 4px 16px rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .switch-header {
  color: #64748b;
  border-bottom: 1px solid rgba(59, 130, 246, 0.15);
}

:root[data-theme="light"] .cluster-item:hover {
  background: rgba(59, 130, 246, 0.1);
}

:root[data-theme="light"] .cluster-item.active {
  background: rgba(59, 130, 246, 0.15);
  border-left: 2px solid #3b82f6;
}

:root[data-theme="light"] .status-dot.connected {
  background: #22c55e;
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
}

:root[data-theme="light"] .status-dot.disconnected {
  background: #94a3b8;
  box-shadow: none;
}

:root[data-theme="light"] .status-text {
  color: #64748b;
}

:root[data-theme="light"] .loading-spinner,
:root[data-theme="light"] .loading-spinner-right {
  color: #3b82f6;
}

:root[data-theme="light"] .cluster-icon {
  color: #3b82f6;
}

:root[data-theme="light"] .cluster-name {
  color: #1e293b;
}

:root[data-theme="light"] .disconnect-btn {
  color: #ef4444;
}

:root[data-theme="light"] .disconnect-btn:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #dc2626;
}

:root[data-theme="light"] .empty-list {
  color: #64748b;
}

:root[data-theme="light"] .switch-footer {
  border-top: 1px solid rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

:root[data-theme="light"] .switch-footer:hover {
  background: rgba(59, 130, 246, 0.1);
}

:root[data-theme="light"] .footer-text {
  color: #3b82f6;
}
</style>
<!-- src/components/cluster/ClusterTabBar.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTabStore, type ClusterTab } from '@/stores/tabs'
import { useClusterStore } from '@/stores/cluster'
import { useMessageStore } from '@/stores/message'
import { useClusterDrag } from '@/composables/useClusterDrag'
import { focusClusterWindow } from '@/utils/window'
import { X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'

const { t } = useI18n()
const tabStore = useTabStore()
const clusterStore = useClusterStore()
const messageStore = useMessageStore()

// 确认弹窗状态
const showConfirmDialog = ref(false)
const pendingCloseTab = ref<ClusterTab | null>(null)

// 使用拖拽 composable
const { handleDragStart, handleDragEnd } = useClusterDrag(
  computed(() => tabStore.clusterTabs),
  {
    onDragOut: (tab: ClusterTab) => {
      // 拖出后移除当前窗口的 Tab
      tabStore.removeClusterTab(tab.id)
    }
  }
)

// 处理集群 Tab 点击
const handleClusterClick = async (tab: ClusterTab) => {
  if (tabStore.activeClusterTabId === tab.id) return

  // 检查是否在其他窗口打开
  const focused = await focusClusterWindow(tab.clientId)
  if (!focused) {
    // 不在其他窗口，切换到该 Tab
    tabStore.switchClusterTab(tab.id)
  }
}

// 处理关闭集群 Tab
const handleCloseCluster = (tab: ClusterTab, event: Event) => {
  event.stopPropagation()
  pendingCloseTab.value = tab
  showConfirmDialog.value = true
}

// 确认关闭
const handleConfirmClose = async () => {
  if (pendingCloseTab.value) {
    await closeClusterTab(pendingCloseTab.value)
    pendingCloseTab.value = null
  }
}

// 取消关闭
const handleCancelClose = () => {
  pendingCloseTab.value = null
}

async function closeClusterTab(tab: ClusterTab) {
  const clientId = tab.clientId

  console.log('[ClusterTabBar] Closing cluster tab:', tab.clusterName, 'clientId:', clientId)

  // 1. 停止该集群所有消息 tab 的实时消费
  const messageTabs = tab.contentTabs.filter(t => t.type === 'message')
  for (const msgTab of messageTabs) {
    const topicName = msgTab.params.topicName
    // 检查是否正在消费这个 topic
    if (messageStore.streamTopic === topicName) {
      try {
        console.log('[ClusterTabBar] Stopping realtime consume for topic:', topicName)
        await messageStore.stopRealtimeConsume(clientId, topicName)
      } catch (e) {
        console.error('[ClusterTabBar] Failed to stop streaming:', e)
      }
    }
  }

  // 2. 断开 Kafka 连接（后端会自动停止所有流式消费会话）
  try {
    console.log('[ClusterTabBar] Disconnecting cluster:', clientId)
    await clusterStore.disconnectCluster(clientId)
    console.log('[ClusterTabBar] Cluster disconnected')
  } catch (e) {
    console.error('[ClusterTabBar] Failed to disconnect:', e)
  }

  // 3. 移除集群 Tab
  tabStore.removeClusterTab(tab.id)
  console.log('[ClusterTabBar] Tab removed')
}
</script>

<template>
  <div class="cluster-tab-bar">
    <div class="cluster-tabs">
      <div
        v-for="tab in tabStore.clusterTabs"
        :key="tab.id"
        class="cluster-tab"
        :class="{ active: tabStore.activeClusterTabId === tab.id }"
        draggable="true"
        @click="handleClusterClick(tab)"
        @dragstart="handleDragStart(tab.id, $event as DragEvent)"
        @dragend="handleDragEnd($event as DragEvent)"
      >
        <span class="status-dot" :class="tab.connected ? 'connected' : 'disconnected'"></span>
        <span class="cluster-name">{{ tab.clusterName }}</span>
        <X
          :size="12"
          class="close-btn"
          @click="handleCloseCluster(tab, $event)"
        />
      </div>
    </div>

    <!-- Confirm Dialog -->
    <ConfirmDialog
      :show="showConfirmDialog"
      :title="t('cluster.confirmClose')"
      :message="t('cluster.closeConfirmText', { name: pendingCloseTab?.clusterName || '' })"
      :confirm-text="t('common.confirm')"
      :cancel-text="t('common.cancel')"
      @confirm="handleConfirmClose"
      @cancel="handleCancelClose"
      @close="showConfirmDialog = false"
    />
  </div>
</template>

<style scoped>
.cluster-tab-bar {
  display: flex;
  justify-content: center;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border);
  height: 36px;
}

.cluster-tabs {
  display: flex;
  width: 100%;
  justify-content: center;
}

.cluster-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  flex: 1;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  border-right: 1px solid var(--border);
  transition: all 0.15s ease;
  position: relative;
  min-width: 0;
}

.cluster-tab:last-child {
  border-right: none;
}

.cluster-tab:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.cluster-tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent);
}

.cluster-tab:not(.active) {
  opacity: 0.8;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected {
  background: var(--success);
}

.status-dot.disconnected {
  background: var(--error);
}

.cluster-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.close-btn {
  color: var(--text-muted);
  padding: 2px;
  border-radius: 3px;
  transition: all 0.15s ease;
  cursor: pointer;
  opacity: 0;
  flex-shrink: 0;
}

.cluster-tab:hover .close-btn {
  opacity: 1;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
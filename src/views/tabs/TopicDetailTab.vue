<!-- src/views/tabs/TopicDetailTab.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useClusterStore } from '@/stores/cluster'
import { useTopicStore } from '@/stores/topic'
import { useTabStore, type Tab } from '@/stores/tabs'
import { useMessageStore } from '@/stores/message'
import IconButton from '@/components/common/IconButton.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import TopicForm from '@/components/topic/TopicForm.vue'
import { FileText, RefreshCw } from 'lucide-vue-next'

const props = defineProps<{
  tab: Tab
}>()

const { t } = useI18n()
const message = useMessage()
const clusterStore = useClusterStore()
const topicStore = useTopicStore()
const tabStore = useTabStore()
const messageStore = useMessageStore()

const loading = ref(false)
const showEditModal = ref(false)
const showDeleteConfirm = ref(false)
const editingTopic = ref<{ name: string; partitions: number } | null>(null)

const topicInfo = ref(topicStore.selectedTopic)

// Calculate total messages from all partitions (high_watermark - low_watermark)
const totalMessages = computed(() => {
  if (!topicInfo.value?.partitions) return 0
  return topicInfo.value.partitions.reduce((sum, p) => {
    const messages = (p.high_watermark || 0) - (p.low_watermark || 0)
    return sum + Math.max(0, messages)
  }, 0)
})

// Format number with commas
const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M'
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K'
  }
  return num.toString()
}

const loadTopicInfo = async () => {
  if (!clusterStore.activeClusterId || !props.tab.params.topicName) return
  loading.value = true
  try {
    await topicStore.getTopicInfo(clusterStore.activeClusterId, props.tab.params.topicName)
    topicInfo.value = topicStore.selectedTopic
  } finally {
    loading.value = false
  }
}

onMounted(loadTopicInfo)

watch(() => props.tab.params.topicName, () => {
  loadTopicInfo()
})

const handleViewMessages = () => {
  tabStore.openTab('message', `${props.tab.params.topicName} 消息`, 'message', {
    topicName: props.tab.params.topicName
  })
}

const handleSendTest = () => {
  messageStore.openProduceDialog(props.tab.params.topicName)
}

const handleEdit = () => {
  if (topicInfo.value) {
    editingTopic.value = {
      name: props.tab.params.topicName,
      partitions: topicInfo.value.partitions.length
    }
    showEditModal.value = true
  }
}

const handleEditSuccess = () => {
  showEditModal.value = false
  editingTopic.value = null
  loadTopicInfo()
}

const handleDelete = () => {
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  if (!clusterStore.activeClusterId || !props.tab.params.topicName) return

  const topicName = props.tab.params.topicName
  const clientId = tabStore.activeClientId

  try {
    // 1. 如果正在实时消费该 topic，停止消费
    if (clientId && messageStore.streamTopic === topicName && messageStore.isStreaming) {
      try {
        await messageStore.stopRealtimeConsume(clientId, topicName)
      } catch (e) {
        console.error('Failed to stop realtime consume:', e)
      }
    }

    // 2. 删除 Topic
    await topicStore.deleteTopic(clusterStore.activeClusterId, topicName)
    message.success(t('topic.deleteSuccess'))

    // 3. 关闭所有集群中与该 topic 相关的消息 Tab 和发送 Tab
    for (const clusterTab of tabStore.clusterTabs) {
      const tabsToClose = clusterTab.contentTabs.filter(t =>
        (t.type === 'message' || t.type === 'send-test') &&
        t.params.topicName === topicName
      )

      for (const tabToClose of tabsToClose) {
        // 如果是当前激活的集群，使用 closeTab
        if (clusterTab.id === tabStore.activeClusterTabId) {
          tabStore.closeTab(tabToClose.id)
        } else {
          // 其他集群，直接从数组中移除
          const index = clusterTab.contentTabs.findIndex(t => t.id === tabToClose.id)
          if (index !== -1) {
            clusterTab.contentTabs.splice(index, 1)
          }
        }
      }
    }

    // 4. 关闭当前 Topic 详情 Tab
    tabStore.closeTab(props.tab.id)
  } catch (e) {
    message.error(t('topic.deleteFailed') + ': ' + String(e))
  } finally {
    showDeleteConfirm.value = false
  }
}
</script>

<template>
  <div class="topic-detail">
    <!-- Full Page Loading Overlay -->
    <div v-if="loading" class="loading-overlay">
      <RefreshCw :size="40" :stroke-width="1.5" class="loading-spinner" />
      <span class="loading-text">{{ t('common.loading') }}</span>
    </div>

    <!-- Content -->
    <template v-else>
      <!-- Header -->
      <div class="detail-header">
        <div class="header-title">
          <FileText :size="20" :stroke-width="1.5" class="title-icon" />
          <span class="title-text">{{ tab.params.topicName }}</span>
        </div>
        <div class="header-actions">
          <IconButton icon="refresh" :tooltip="t('tooltip.refresh')" size="small" @click="loadTopicInfo" />
          <IconButton icon="message" :tooltip="t('tooltip.viewMessages')" size="small" @click="handleViewMessages" />
          <IconButton icon="send" :tooltip="t('tooltip.sendMessage')" size="small" @click="handleSendTest" />
          <IconButton icon="edit" :tooltip="t('tooltip.edit')" size="small" @click="handleEdit" />
          <IconButton icon="delete" :tooltip="t('tooltip.delete')" danger size="small" @click="handleDelete" />
        </div>
      </div>

      <!-- Stats Cards -->
      <div class="stats-row">
        <div class="stat-card">
          <span class="stat-label">{{ t('topic.partitions') }}</span>
          <span class="stat-value">{{ topicInfo?.partitions?.length || 0 }}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">{{ t('topic.replication') }}</span>
          <span class="stat-value">{{ topicInfo?.partitions?.[0]?.replicas?.length || 1 }}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">{{ t('topic.messages') }}</span>
          <span class="stat-value">{{ formatNumber(totalMessages) }}</span>
        </div>
      </div>

      <!-- Partition Table -->
      <div class="partition-table">
        <div class="table-header">
          <span>{{ t('topic.leaderHost') }}</span>
          <span>{{ t('topic.partition') }}</span>
          <span>{{ t('topic.leader') }}</span>
          <span>{{ t('topic.isr') }}</span>
          <span>{{ t('topic.offset') }}</span>
        </div>
        <div
          v-for="partition in topicInfo?.partitions"
          :key="partition.id"
          class="table-row"
        >
          <span>{{ partition.leader_host }}</span>
          <span>{{ partition.id }}</span>
          <span>{{ partition.leader }}</span>
          <span>{{ partition.isr?.join(', ') || '-' }}</span>
          <span>{{ partition.high_watermark }}</span>
        </div>
      </div>
    </template>

    <!-- Edit Modal -->
    <TopicForm
      v-model:show="showEditModal"
      :edit-topic="editingTopic ?? undefined"
      @success="handleEditSuccess"
    />

    <!-- Delete Confirm Modal -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      :title="t('topic.confirmDelete')"
      :message="t('topic.deleteConfirmText', { name: tab.params.topicName })"
      :confirm-text="t('common.confirm')"
      :cancel-text="t('common.cancel')"
      @confirm="confirmDelete"
      @cancel="showDeleteConfirm = false"
      @close="showDeleteConfirm = false"
    />
  </div>
</template>

<style scoped>
.topic-detail {
  position: relative;
  padding: 0;
  min-height: 100%;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.title-icon {
  color: var(--text-primary);
}

.title-text {
  font-size: 18px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.stats-row {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.stat-card {
  flex: 1;
  background: var(--bg-tertiary);
  padding: 14px 18px;
  border-radius: 8px;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.stat-value {
  font-size: 22px;
  color: var(--text-primary);
}

.partition-table {
  background: var(--bg-tertiary);
  border-radius: 8px;
  overflow: hidden;
}

.table-header,
.table-row {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  padding: 12px 16px;
}

.table-header {
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 12px;
  border-bottom: 1px solid var(--border);
}

.table-row {
  color: var(--text-secondary);
  font-size: 13px;
  border-bottom: 1px solid var(--border-light);
}

.table-row:last-child {
  border-bottom: none;
}

.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  background: var(--bg-primary);
  z-index: 10;
}

.loading-spinner {
  color: var(--accent);
  animation: spin 1s linear infinite;
}

.loading-text {
  color: var(--text-muted);
  font-size: 14px;
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
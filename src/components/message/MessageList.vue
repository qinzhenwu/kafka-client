<script setup lang="ts">
import { ref, h, computed, watch, onMounted, onUnmounted } from 'vue'
import {
  NDataTable, NTag,
  NSelect, NInput, NScrollbar, useMessage
} from 'naive-ui'
import type { DataTableColumns, SelectOption } from 'naive-ui'
import { useMessageStore, type KafkaMessage } from '@/stores/message'
import { useClusterStore } from '@/stores/cluster'
import { useTopicStore } from '@/stores/topic'
import { useI18n } from 'vue-i18n'
import IconButton from '@/components/common/IconButton.vue'
import { MessageSquare, Mail, Search, X } from 'lucide-vue-next'

const props = defineProps<{
  topicName: string
}>()

const { t } = useI18n()
const messageStore = useMessageStore()
const clusterStore = useClusterStore()
const topicStore = useTopicStore()
const message = useMessage()

const selectedPartition = ref<number | 'all'>('all')
const startOffset = ref<string>('0')
const messageLimit = ref(200)
const showDetail = ref(false)
const selectedMessage = ref<KafkaMessage | null>(null)

// Real-time consumption controls
const startPosition = ref<'earliest' | 'latest'>('latest')

// Partition count
const partitionCount = ref(0)

// Load topic info to get partition count
const loadPartitionCount = async () => {
  if (!clusterStore.activeClusterId || !props.topicName) return
  try {
    const info = await topicStore.getTopicInfo(clusterStore.activeClusterId, props.topicName)
    partitionCount.value = info?.partitions?.length || 0
  } catch (e) {
    console.error('Failed to load topic info:', e)
  }
}

onMounted(() => {
  // Clear any existing messages when opening a new message tab
  messageStore.clearMessages()
  loadPartitionCount()
})

// Clear messages when topic changes
watch(() => props.topicName, (newTopicName, oldTopicName) => {
  if (oldTopicName && newTopicName !== oldTopicName) {
    // Stop streaming if active
    if (messageStore.isStreaming && clusterStore.activeClusterId) {
      messageStore.stopRealtimeConsume(clusterStore.activeClusterId, oldTopicName)
    }
    // Clear old messages
    messageStore.clearMessages()
    // Reload partition count for new topic
    loadPartitionCount()
  }
})

// Partition options for dropdown
const partitionOptions = computed<SelectOption[]>(() => {
  const options: SelectOption[] = [
    { label: t('message.allPartitions', '全部'), value: 'all' }
  ]
  for (let i = 0; i < partitionCount.value; i++) {
    options.push({ label: String(i), value: i })
  }
  return options
})

// Limit options
const limitOptions = [
  { label: '100', value: 100 },
  { label: '200', value: 200 },
  { label: '500', value: 500 },
  { label: '1000', value: 1000 },
]

// Search
const searchKeyword = ref('')

const filteredMessages = computed(() => {
  if (!searchKeyword.value) return messageStore.messages
  const keyword = searchKeyword.value.toLowerCase()
  return messageStore.messages.filter(msg =>
    msg.key?.toLowerCase().includes(keyword) ||
    msg.value?.toLowerCase().includes(keyword)
  )
})

// Format timestamp to yyyy-MM-dd HH:mm:ss
const formatTimestamp = (timestamp: string): string => {
  const date = new Date(timestamp)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}

const columns = computed<DataTableColumns<KafkaMessage>>(() => [
  {
    title: t('message.partition'),
    key: 'partition',
    width: 80,
    render: (row) => h(NTag, { size: 'small' }, () => row.partition),
  },
  {
    title: t('message.offset'),
    key: 'offset',
    width: 100,
  },
  {
    title: t('message.key'),
    key: 'key',
    width: 150,
    ellipsis: { tooltip: true },
    render: (row) => row.key || '-',
  },
  {
    title: t('message.value'),
    key: 'value',
    ellipsis: { tooltip: true },
    render: (row) => {
      const value = row.value || '-'
      if (value.length > 100) {
        return value.substring(0, 100) + '...'
      }
      return value
    },
  },
  {
    title: t('message.timestamp'),
    key: 'timestamp',
    width: 180,
    render: (row) => formatTimestamp(row.timestamp),
  },
  {
    title: t('common.action'),
    key: 'actions',
    width: 80,
    render: (row) =>
      h(IconButton, {
        icon: 'view',
        tooltip: t('message.view'),
        size: 'small',
        onClick: () => viewDetail(row)
      }),
  },
])

const viewDetail = (msg: KafkaMessage) => {
  selectedMessage.value = msg
  showDetail.value = true
}

const handleConsume = async () => {
  if (!clusterStore.activeClusterId) return

  // Stop streaming if active
  if (messageStore.isStreaming) {
    await messageStore.stopRealtimeConsume(
      clusterStore.activeClusterId,
      props.topicName
    )
  }

  // Clear existing messages before loading new ones
  messageStore.clearMessages()

  try {
    const offsetNum = parseInt(startOffset.value, 10) || 0
    const partitionNum = selectedPartition.value === 'all' ? null : selectedPartition.value

    if (partitionNum !== null && offsetNum >= 0) {
      await messageStore.consumeFromOffset(
        clusterStore.activeClusterId,
        props.topicName,
        partitionNum,
        offsetNum,
        messageLimit.value
      )
    } else {
      await messageStore.consumeFromPosition(
        clusterStore.activeClusterId,
        props.topicName,
        startPosition.value,
        messageLimit.value
      )
    }
    message.success(t('message.consumedMessages', { count: messageStore.messages.length }))
  } catch (e: unknown) {
    message.error(t('message.consumeError') + ': ' + String(e))
  }
}

const handleStartStreaming = async () => {
  if (!clusterStore.activeClusterId) return

  // Clear existing messages before starting streaming
  messageStore.clearMessages()

  try {
    await messageStore.startRealtimeConsume(
      clusterStore.activeClusterId,
      props.topicName,
      startPosition.value
    )
    message.success(t('message.startedStreaming'))
  } catch (e: unknown) {
    message.error(t('message.streamingError') + ': ' + String(e))
  }
}

const handleStopStreaming = async () => {
  if (!clusterStore.activeClusterId) return

  try {
    await messageStore.stopRealtimeConsume(
      clusterStore.activeClusterId,
      props.topicName
    )
    message.success(t('message.stoppedStreaming'))
  } catch (e: unknown) {
    message.error(t('message.streamingError') + ': ' + String(e))
  }
}

const formatJsonValue = (value: string | null): string => {
  if (!value) return ''
  try {
    return JSON.stringify(JSON.parse(value), null, 2)
  } catch {
    return value
  }
}

const formatXmlValue = (value: string | null): string => {
  if (!value) return ''
  try {
    // Simple XML formatting
    let formatted = value
    let indent = ''
    for (let i = 0; i < value.length; i++) {
      if (value[i] === '>' && value[i + 1] !== '<') {
        formatted = formatted.substring(0, i + 1) + '\n' + indent
      } else if (value[i] === '<' && value[i + 1] === '/') {
        indent = indent.slice(0, -2)
        formatted = formatted.substring(0, i) + '\n' + indent + formatted.substring(i)
        i += indent.length + 1
      } else if (value[i] === '>' && value[i + 1] === '<' && value[i + 2] !== '/') {
        indent += '  '
        formatted = formatted.substring(0, i + 1) + '\n' + indent + formatted.substring(i + 1)
        i += indent.length + 1
      }
    }
    return formatted
  } catch {
    return value
  }
}

const isJson = (value: string | null): boolean => {
  if (!value) return false
  try {
    JSON.parse(value)
    return true
  } catch {
    return false
  }
}

const isXml = (value: string | null): boolean => {
  if (!value) return false
  return value.trim().startsWith('<') && value.trim().endsWith('>')
}

const getContentType = (value: string | null): 'json' | 'xml' | 'text' => {
  if (isJson(value)) return 'json'
  if (isXml(value)) return 'xml'
  return 'text'
}

const getFormattedValue = (value: string | null): string => {
  if (!value) return ''
  if (isJson(value)) return formatJsonValue(value)
  if (isXml(value)) return formatXmlValue(value)
  return value
}

// Clean up event listener on unmount
onUnmounted(() => {
  if (messageStore.isStreaming && clusterStore.activeClusterId) {
    messageStore.stopRealtimeConsume(clusterStore.activeClusterId, props.topicName)
  }
})
</script>

<template>
  <div class="message-list">
    <!-- Header -->
    <div class="list-header">
      <div class="header-title">
        <MessageSquare :size="20" :stroke-width="1.5" class="title-icon" />
        <span class="title-text">{{ topicName }}</span>
        <span v-if="messageStore.isStreaming" class="streaming-badge">● {{ t('message.streaming') }}</span>
      </div>
      <div class="header-actions">
        <IconButton
          v-if="!messageStore.isStreaming"
          icon="consume"
          :tooltip="t('message.consume')"
          size="small"
          :loading="messageStore.loading"
          :disabled="messageStore.loading"
          @click="handleConsume"
        />
        <IconButton
          v-if="!messageStore.isStreaming"
          icon="streamStart"
          :tooltip="t('message.startStreaming')"
          size="small"
          :disabled="messageStore.loading"
          @click="handleStartStreaming"
        />
        <IconButton
          v-if="messageStore.isStreaming"
          icon="streamStop"
          :tooltip="t('message.stopStreaming')"
          size="small"
          danger
          filled
          @click="handleStopStreaming"
        />
        <IconButton
          icon="clear"
          :tooltip="t('message.clear')"
          size="small"
          danger
          :disabled="messageStore.loading"
          @click="messageStore.clearMessages"
        />
      </div>
    </div>

    <!-- Filters -->
    <div class="filter-row">
      <div class="filter-group">
        <label>{{ t('message.startPosition') }}</label>
        <n-select
          v-model:value="startPosition"
          :options="[
            { label: t('message.earliest'), value: 'earliest' },
            { label: t('message.latest'), value: 'latest' }
          ]"
          size="small"
          style="width: 100px"
        />
      </div>

      <div class="filter-group">
        <label>{{ t('message.partition') }}</label>
        <n-select
          v-model:value="selectedPartition"
          :options="partitionOptions"
          size="small"
          style="width: 100px"
        />
      </div>

      <div class="filter-group">
        <label>{{ t('message.offset') }}</label>
        <n-input
          v-model:value="startOffset"
          placeholder="0"
          size="small"
          style="width: 100px"
          @input="(val: string) => startOffset = val.replace(/[^\d]/g, '')"
        />
      </div>

      <div class="filter-group">
        <label>{{ t('message.limit') }}</label>
        <n-select
          v-model:value="messageLimit"
          :options="limitOptions"
          size="small"
          style="width: 100px"
        />
      </div>

      <div class="filter-spacer"></div>

      <div class="filter-group">
        <n-input
          v-model:value="searchKeyword"
          :placeholder="t('message.searchMessages')"
          clearable
          size="small"
          style="width: 200px"
        >
          <template #prefix>
            <Search :size="14" :stroke-width="1.5" style="opacity: 0.5" />
          </template>
        </n-input>
        <IconButton
          icon="search"
          :tooltip="t('message.query')"
          size="small"
          :loading="messageStore.loading"
          :disabled="messageStore.loading || messageStore.isStreaming"
          @click="handleConsume"
        />
      </div>
    </div>

    <!-- Stats -->
    <div class="stats-row">
      <span class="stat-item">{{ t('message.messageCount') }}: <strong>{{ messageStore.messages.length }}</strong></span>
    </div>

    <!-- Data Table -->
    <div class="table-wrapper">
      <n-data-table
        :columns="columns"
        :data="filteredMessages"
        :pagination="false"
        :row-key="(row: KafkaMessage) => `${row.partition}-${row.offset}`"
        max-height="calc(100vh - 350px)"
        virtual-scroll
      />
    </div>

    <!-- Detail Modal -->
    <div v-if="showDetail" class="modal-overlay" @click="showDetail = false">
      <div class="modal-container detail-modal" @click.stop>
        <!-- Header -->
        <div class="modal-header">
          <span class="header-title">
            <Mail :size="18" :stroke-width="1.5" class="header-icon" />
            {{ t('message.detail') }}
          </span>
          <X :size="16" class="header-close" @click="showDetail = false" />
        </div>

        <!-- Content -->
        <div class="modal-content" v-if="selectedMessage">
          <div class="detail-info">
            <div class="info-item">
              <span class="info-label">{{ t('message.partition') }}</span>
              <span class="info-value">{{ selectedMessage.partition }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('message.offset') }}</span>
              <span class="info-value">{{ selectedMessage.offset }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('message.timestamp') }}</span>
              <span class="info-value">{{ formatTimestamp(selectedMessage.timestamp) }}</span>
            </div>
          </div>

          <div class="detail-section">
            <div class="section-label">{{ t('message.key') }}</div>
            <div class="content-box">
              <n-scrollbar style="max-height: 120px">
                <pre class="content-text">{{ selectedMessage.key || '' }}</pre>
              </n-scrollbar>
            </div>
          </div>

          <div class="detail-section">
            <div class="section-label">
              {{ t('message.value') }}
              <n-tag v-if="getContentType(selectedMessage.value) !== 'text'" size="small" type="info" style="margin-left: 8px">
                {{ getContentType(selectedMessage.value).toUpperCase() }}
              </n-tag>
            </div>
            <div class="content-box value-box">
              <n-scrollbar style="max-height: 400px">
                <pre class="content-text">{{ getFormattedValue(selectedMessage.value) || '' }}</pre>
              </n-scrollbar>
            </div>
          </div>

          <div v-if="Object.keys(selectedMessage.headers).length > 0" class="detail-section">
            <div class="section-label">{{ t('message.headers') }}</div>
            <div class="headers-list">
              <div v-for="(value, key) in selectedMessage.headers" :key="key" class="header-item">
                <span class="header-key">{{ key }}</span>
                <span class="header-value">{{ value }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="modal-footer">
          <span class="footer-hint">{{ t('message.detailHint', 'Message details') }}</span>
          <button class="action-btn" @click="showDetail = false">{{ t('common.close', '关闭') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.message-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  overflow: hidden;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.streaming-badge {
  font-size: 12px;
  color: var(--success);
  background: rgba(82, 196, 26, 0.1);
  padding: 2px 8px;
  border-radius: 10px;
}

.header-actions {
  display: flex;
  gap: 4px;
}

.filter-row {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.filter-group label {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.filter-spacer {
  flex: 1;
}

.stats-row {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--text-muted);
}

.stat-item strong {
  color: var(--text-primary);
}

.table-wrapper {
  flex: 1;
  min-height: 0;
}

.table-wrapper :deep(.n-data-table) {
  height: 100%;
}

.table-wrapper :deep(.n-data-table-wrapper) {
  height: 100%;
}

.table-wrapper :deep(.n-data-table-base-table-body) {
  overflow-y: auto !important;
}

.detail-info {
  display: flex;
  gap: 24px;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 11px;
  color: var(--text-muted);
}

.info-value {
  font-size: 14px;
  color: var(--text-primary);
}

.detail-section {
  margin-top: 16px;
}

.section-label {
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  font-size: 14px;
}

.content-box {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px 16px;
  transition: all 0.2s ease;
}

.content-box:hover {
  border-color: var(--accent);
}

.value-box {
  background: var(--bg-secondary);
}

.content-text {
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.headers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}

.header-item {
  display: flex;
  gap: 8px;
  padding: 6px 10px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 13px;
}

.header-key {
  color: var(--accent);
  font-weight: 500;
}

.header-value {
  color: var(--text-secondary);
}

/* Naive UI overrides */
:deep(.n-data-table) {
  background: var(--bg-secondary);
  border-radius: 8px;
  overflow: hidden;
}

:deep(.n-data-table-th) {
  background: var(--bg-tertiary) !important;
  color: var(--text-muted) !important;
  font-size: 12px;
  font-weight: 500;
}

:deep(.n-data-table-td) {
  background: var(--bg-secondary) !important;
  color: var(--text-secondary) !important;
  font-size: 13px;
}

:deep(.n-data-table-tr:hover .n-data-table-td) {
  background: var(--bg-hover) !important;
}

:deep(.n-data-table-empty) {
  background: var(--bg-secondary) !important;
}

:deep(.n-card) {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
}

:deep(.n-card-header) {
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border);
}

:deep(.n-card-header .n-card-header__main) {
  color: var(--text-primary);
  font-weight: 500;
}

:deep(.n-tag) {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  width: 700px;
  max-width: 95vw;
  max-height: 90vh;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-container .modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.modal-container .header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.modal-container .header-icon {
  color: var(--text-primary);
}

.modal-container .header-close {
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.modal-container .header-close:hover {
  color: var(--text-primary);
}

.modal-container .modal-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.modal-container .modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.modal-container .footer-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.modal-container .action-btn {
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.modal-container .action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
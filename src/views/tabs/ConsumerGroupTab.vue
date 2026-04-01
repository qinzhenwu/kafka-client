<!-- src/views/tabs/ConsumerGroupTab.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useClusterStore } from '@/stores/cluster'
import { useConsumerStore, type LagInfo } from '@/stores/consumer'
import { type Tab } from '@/stores/tabs'
import IconButton from '@/components/common/IconButton.vue'
import ResetOffsetsModal from '@/components/consumer/ResetOffsetsModal.vue'
import { Users, RefreshCw } from 'lucide-vue-next'

const props = defineProps<{
  tab: Tab
}>()

const { t } = useI18n()
const message = useMessage()
const clusterStore = useClusterStore()
const consumerStore = useConsumerStore()

const loading = ref(false)
const showResetModal = ref(false)

// Use local state instead of global store state
const localGroupInfo = ref(consumerStore.selectedGroup)
const localLagInfo = ref<LagInfo[]>([])

// Use local state for computed
const groupInfo = computed(() => localGroupInfo.value)
const lagInfo = computed(() => localLagInfo.value)

// Compute total lag
const totalLag = computed(() => {
  return lagInfo.value.reduce((sum, l) => sum + Math.max(0, l.lag), 0)
})

// Group lag by topic
const lagByTopic = computed(() => {
  const grouped = new Map<string, LagInfo[]>()
  for (const lag of lagInfo.value) {
    if (!grouped.has(lag.topic)) {
      grouped.set(lag.topic, [])
    }
    grouped.get(lag.topic)!.push(lag)
  }
  return grouped
})

// Format number
const formatNumber = (num: number): string => {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

// State color
const stateColor = computed(() => {
  const state = groupInfo.value?.state || ''
  if (state === 'Stable') return 'var(--success)'
  if (state === 'Empty') return 'var(--text-muted)'
  if (state === 'Rebalancing') return 'var(--warning)'
  return 'var(--error)'
})

// Topics subscribed by this group (for reset modal)
const subscribedTopics = computed(() => {
  const topics = new Set<string>()
  for (const lag of lagInfo.value) {
    topics.add(lag.topic)
  }
  return Array.from(topics)
})

// Partition counts per topic (for reset modal)
const partitionCounts = computed(() => {
  const counts: Record<string, number> = {}
  for (const [topic, lags] of lagByTopic.value) {
    counts[topic] = lags.length
  }
  return counts
})

const loadData = async () => {
  if (!clusterStore.activeClusterId || !props.tab.params.groupId) return
  loading.value = true
  try {
    const info = await consumerStore.getGroupInfo(clusterStore.activeClusterId, props.tab.params.groupId)
    localGroupInfo.value = info
    const lag = await consumerStore.getGroupLag(clusterStore.activeClusterId, props.tab.params.groupId)
    localLagInfo.value = lag
  } catch (e: unknown) {
    message.error(String(e))
  } finally {
    loading.value = false
  }
}

onMounted(loadData)

watch(() => props.tab.params.groupId, (newId, oldId) => {
  if (newId && newId !== oldId) {
    loadData()
  }
})
</script>

<template>
  <div class="consumer-group-detail">
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
          <Users :size="20" :stroke-width="1.5" class="title-icon" />
          <span class="title-text">{{ tab.params.groupId }}</span>
        </div>
        <div class="header-actions">
          <IconButton icon="refresh" :tooltip="t('tooltip.refresh')" size="small" @click="loadData" />
          <IconButton icon="reset" :tooltip="t('consumer.resetOffsets')" size="small" @click="showResetModal = true" />
        </div>
      </div>

      <!-- Stats Row -->
      <div class="stats-row">
        <div class="stat-card">
          <span class="stat-label">{{ t('consumer.state') }}</span>
          <span class="stat-value" :style="{ color: stateColor }">{{ groupInfo?.state || '-' }}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">{{ t('consumer.memberCount') }}</span>
          <span class="stat-value">{{ groupInfo?.members?.length || 0 }}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">{{ t('consumer.totalLag') }}</span>
          <span class="stat-value">{{ formatNumber(totalLag) }}</span>
        </div>
      </div>

      <!-- Members Section -->
      <div class="section">
        <h3 class="section-title">{{ t('consumer.members') }}</h3>
        <div v-if="!groupInfo?.members?.length" class="empty-state">
          {{ t('consumer.noMembers') }}
        </div>
        <div v-else class="members-table">
          <div class="table-header">
            <span>{{ t('consumer.clientId') }}</span>
            <span>{{ t('consumer.host') }}</span>
            <span>{{ t('consumer.partitions') }}</span>
          </div>
          <div v-for="member in groupInfo?.members" :key="member.member_id" class="table-row">
            <span>{{ member.client_id }}</span>
            <span>{{ member.client_host }}</span>
            <span class="partition-chips">
              <span v-for="p in member.assignment" :key="p" class="partition-chip">{{ p }}</span>
              <span v-if="!member.assignment?.length" class="no-assignment">-</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Lag Section -->
      <div class="section">
        <h3 class="section-title">{{ t('consumer.lagByTopic') }}</h3>
        <div v-if="!lagInfo.length" class="empty-state">
          {{ t('consumer.noLag') }}
        </div>
        <div v-else class="lag-section">
          <div v-for="[topic, lags] in lagByTopic" :key="topic" class="topic-group">
            <div class="topic-header">
              <span class="topic-name">{{ topic }}</span>
              <span class="topic-lag">{{ formatNumber(lags.reduce((s, l) => s + Math.max(0, l.lag), 0)) }}</span>
            </div>
            <div class="lag-table">
              <div class="table-header">
                <span>{{ t('consumer.partitions') }}</span>
                <span>{{ t('consumer.currentOffset') }}</span>
                <span>{{ t('consumer.endOffset') }}</span>
                <span>{{ t('consumer.lag') }}</span>
              </div>
              <div v-for="lag in lags" :key="lag.partition" class="table-row">
                <span>{{ lag.partition }}</span>
                <span>{{ lag.current_offset }}</span>
                <span>{{ lag.end_offset }}</span>
                <span :class="{ 'lag-high': lag.lag > 1000 }">{{ formatNumber(lag.lag) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Reset Modal -->
    <ResetOffsetsModal
      v-model:show="showResetModal"
      :group-id="tab.params.groupId"
      :topics="subscribedTopics"
      :partition-counts="partitionCounts"
      @success="loadData"
    />
  </div>
</template>

<style scoped>
.consumer-group-detail {
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
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
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
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  padding: 14px 18px;
  border-radius: 12px;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.stat-value {
  font-size: 22px;
  color: var(--text-primary);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0 0 12px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.empty-state {
  color: var(--text-muted);
  font-size: 13px;
  padding: 24px;
  text-align: center;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-radius: 12px;
}

.members-table,
.lag-table {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  overflow: hidden;
}

.table-header,
.table-row {
  display: grid;
  padding: 12px 16px;
}

.members-table .table-header,
.members-table .table-row {
  grid-template-columns: 1fr 1fr 1fr;
}

.partition-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.partition-chip {
  background: var(--accent-bg);
  color: var(--accent);
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
}

.no-assignment {
  color: var(--text-muted);
}

.lag-table .table-header,
.lag-table .table-row {
  grid-template-columns: 80px 1fr 1fr 100px;
}

.table-header {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: var(--text-muted);
  font-size: 12px;
  border-bottom: 1px solid var(--glass-border);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.table-row {
  color: var(--text-secondary);
  font-size: 13px;
  border-bottom: 1px solid var(--glass-border);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.table-row:last-child {
  border-bottom: none;
}

.topic-group {
  margin-bottom: 12px;
}

.topic-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-bottom: none;
  border-radius: 12px 12px 0 0;
}

.topic-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.topic-lag {
  font-size: 13px;
  color: var(--accent);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.topic-group .lag-table {
  border-radius: 0 0 12px 12px;
  border: 1px solid var(--glass-border);
  border-top: none;
}

.lag-high {
  color: var(--warning);
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

/* Light Mode */
:root[data-theme="light"] .consumer-group-detail {
  background: transparent;
}

:root[data-theme="light"] .title-icon {
  color: #1e293b;
}

:root[data-theme="light"] .title-text {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .stat-card {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .stat-label {
  color: #64748b;
  text-shadow: none;
}

:root[data-theme="light"] .stat-value {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .section-title {
  color: #475569;
  text-shadow: none;
}

:root[data-theme="light"] .empty-state {
  color: #64748b;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .members-table,
:root[data-theme="light"] .lag-table {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .table-header {
  background: rgba(0, 0, 0, 0.03);
  color: #64748b;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  text-shadow: none;
}

:root[data-theme="light"] .table-row {
  color: #475569;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  text-shadow: none;
}

:root[data-theme="light"] .partition-chip {
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}

:root[data-theme="light"] .no-assignment {
  color: #64748b;
}

:root[data-theme="light"] .topic-header {
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .topic-name {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .topic-lag {
  color: #3b82f6;
  text-shadow: none;
}

:root[data-theme="light"] .topic-group .lag-table {
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .loading-overlay {
  background: transparent;
}

:root[data-theme="light"] .loading-text {
  color: #64748b;
}
</style>
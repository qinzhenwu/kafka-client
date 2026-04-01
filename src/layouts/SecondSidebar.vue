<!-- src/layouts/SecondSidebar.vue -->
<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { NInput } from 'naive-ui'
import { useClusterStore } from '@/stores/cluster'
import { useTopicStore } from '@/stores/topic'
import { useConsumerStore } from '@/stores/consumer'
import { useTabStore } from '@/stores/tabs'
import IconButton from '@/components/common/IconButton.vue'
import TopicForm from '@/components/topic/TopicForm.vue'
import { FileText, Users, Search, Link, RefreshCw } from 'lucide-vue-next'

const { t } = useI18n()
const clusterStore = useClusterStore()
const topicStore = useTopicStore()
const consumerStore = useConsumerStore()
const tabStore = useTabStore()

defineProps<{
  activeNav: string
}>()

const emit = defineEmits<{
  selectTopic: [topicName: string]
  selectConsumerGroup: [groupId: string]
}>()

const searchQuery = ref('')
const groupSearchQuery = ref('')
const showCreateModal = ref(false)
const isLoading = ref(false)
const isRefreshingTopics = ref(false)
const isRefreshingGroups = ref(false)

// Resizable sidebar
const sidebarRef = ref<HTMLElement | null>(null)
const sidebarWidth = ref(200)
const isResizing = ref(false)
const minWidth = 150
const maxWidth = 400

const hasActiveCluster = computed(() => !!tabStore.activeClientId)

// Sort topics case-sensitively (uppercase before lowercase)
const sortedTopics = computed(() => {
  return [...topicStore.topics].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'case' }))
})

// Sort consumer groups case-sensitively
const sortedGroups = computed(() => {
  return [...consumerStore.groups].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'case' }))
})

const filteredTopics = computed(() => {
  const source = sortedTopics.value
  if (!searchQuery.value) return source
  const query = searchQuery.value.toLowerCase()
  return source.filter(name =>
    name.toLowerCase().includes(query)
  )
})

const filteredGroups = computed(() => {
  const source = sortedGroups.value
  if (!groupSearchQuery.value) return source
  const query = groupSearchQuery.value.toLowerCase()
  return source.filter(name =>
    name.toLowerCase().includes(query)
  )
})

const selectedTopicInList = computed(() => {
  if (!tabStore.activeTab || tabStore.activeTab.type !== 'topic') return null
  return tabStore.activeTab.params.topicName
})

const selectedGroupInList = computed(() => {
  if (!tabStore.activeTab || tabStore.activeTab.type !== 'consumer-group') return null
  return tabStore.activeTab.params.groupId
})

// Load topics when cluster is active
watch(() => tabStore.activeClusterTabId, async (clusterTabId, oldClusterTabId) => {
  console.log('[SecondSidebar] activeClusterTabId changed:', { oldClusterTabId, clusterTabId })

  const clientId = tabStore.activeClientId
  console.log('[SecondSidebar] Computed activeClientId:', clientId)
  console.log('[SecondSidebar] Current clusterTabs:', tabStore.clusterTabs.map(t => ({ id: t.id, clientId: t.clientId })))

  // 注意：不需要调用 closeAllTabs()！
  // 每个 ClusterTab 有自己的 contentTabs 数组，
  // activeContentTabs 计算属性会自动切换到新集群的内容 Tab 列表。

  if (clientId) {
    console.log('[SecondSidebar] Loading data for clientId:', clientId)
    isLoading.value = true
    try {
      await Promise.all([
        topicStore.listTopics(clientId),
        consumerStore.listGroups(clientId)
      ])
      // Fetch cluster info for the active cluster
      await clusterStore.fetchActiveClusterInfo(clientId)
      console.log('[SecondSidebar] Data loaded successfully')
    } catch (e) {
      console.error('Failed to load data:', e)
    } finally {
      isLoading.value = false
    }
  } else {
    console.log('[SecondSidebar] No active clientId, clearing cluster info')
    // Clear cluster info when no cluster is active
    clusterStore.clearActiveClusterInfo()
  }
}, { immediate: true })

const handleRefreshTopics = async () => {
  if (!tabStore.activeClientId || isRefreshingTopics.value) return
  isRefreshingTopics.value = true
  try {
    await topicStore.listTopics(tabStore.activeClientId)
  } catch (e) {
    console.error('Failed to refresh topics:', e)
  } finally {
    isRefreshingTopics.value = false
  }
}

const handleRefreshGroups = async () => {
  if (!tabStore.activeClientId || isRefreshingGroups.value) return
  isRefreshingGroups.value = true
  try {
    await consumerStore.listGroups(tabStore.activeClientId)
  } catch (e) {
    console.error('Failed to refresh groups:', e)
  } finally {
    isRefreshingGroups.value = false
  }
}

const handleTopicClick = (topicName: string) => {
  emit('selectTopic', topicName)
}

const handleGroupClick = (groupId: string) => {
  emit('selectConsumerGroup', groupId)
}

const handleCreateTopic = () => {
  showCreateModal.value = true
}

const handleCreateSuccess = () => {
  showCreateModal.value = false
}

// Resize handlers
const startResize = (_e: MouseEvent) => {
  isResizing.value = true
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return
  const newWidth = e.clientX - 56 // 56 is icon sidebar width
  sidebarWidth.value = Math.min(maxWidth, Math.max(minWidth, newWidth))
}

const stopResize = () => {
  isResizing.value = false
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onUnmounted(() => {
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<template>
  <div class="second-sidebar" ref="sidebarRef" :style="{ width: sidebarWidth + 'px' }">
    <!-- Loading State -->
    <template v-if="isLoading">
      <div class="loading-state">
        <RefreshCw :size="24" :stroke-width="1.5" class="loading-spinner" />
        <span class="loading-text">{{ t('common.loading') }}</span>
      </div>
    </template>

    <template v-else-if="!hasActiveCluster">
      <div class="empty-state">
        <Link :size="32" :stroke-width="1.5" class="empty-icon" />
        <span class="empty-text">{{ t('empty.noCluster') }}</span>
        <span class="empty-hint">{{ t('empty.noClusterHint') }}</span>
      </div>
    </template>

    <template v-else-if="activeNav === 'topics'">
      <div class="sidebar-section">
        <div class="section-header">
          <IconButton icon="add" :tooltip="t('sidebar.newTopic')" size="small" @click="handleCreateTopic" />
          <button
            class="refresh-btn"
            :title="t('tooltip.refresh')"
            :disabled="isRefreshingTopics"
            @click="handleRefreshTopics"
          >
            <RefreshCw :size="14" :stroke-width="1.5" :class="{ spinning: isRefreshingTopics }" />
          </button>
        </div>

        <div class="search-box">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('sidebar.search')"
            size="small"
            clearable
          >
            <template #prefix>
              <Search :size="12" :stroke-width="1.5" class="search-icon" />
            </template>
          </n-input>
        </div>

        <div class="item-list">
          <div
            v-for="topic in filteredTopics"
            :key="topic"
            class="list-item"
            :class="{ active: selectedTopicInList === topic }"
            @click="handleTopicClick(topic)"
          >
            <FileText :size="14" :stroke-width="1.5" class="item-icon" />
            <span class="item-name">{{ topic }}</span>
          </div>

          <div v-if="filteredTopics.length === 0 && !searchQuery" class="empty-list">
            <span>{{ t('sidebar.noTopics') }}</span>
          </div>
          <div v-else-if="filteredTopics.length === 0 && searchQuery" class="empty-list">
            <span>{{ t('sidebar.noMatch') }}</span>
          </div>
        </div>
      </div>

      <!-- Create Topic Modal -->
      <TopicForm
        v-model:show="showCreateModal"
        @success="handleCreateSuccess"
      />
    </template>

    <template v-else-if="activeNav === 'consumer-groups'">
      <div class="sidebar-section">
        <div class="search-row">
          <div class="search-box">
            <n-input
              v-model:value="groupSearchQuery"
              :placeholder="t('sidebar.search')"
              size="small"
              clearable
            >
              <template #prefix>
                <Search :size="12" :stroke-width="1.5" class="search-icon" />
              </template>
            </n-input>
          </div>
          <button
            class="refresh-btn"
            :title="t('tooltip.refresh')"
            :disabled="isRefreshingGroups"
            @click="handleRefreshGroups"
          >
            <RefreshCw :size="14" :stroke-width="1.5" :class="{ spinning: isRefreshingGroups }" />
          </button>
        </div>

        <div class="item-list">
          <div
            v-for="group in filteredGroups"
            :key="group"
            class="list-item"
            :class="{ active: selectedGroupInList === group }"
            @click="handleGroupClick(group)"
          >
            <Users :size="14" :stroke-width="1.5" class="item-icon" />
            <span class="item-name">{{ group }}</span>
          </div>

          <div v-if="filteredGroups.length === 0 && !groupSearchQuery" class="empty-list">
            <span>{{ t('sidebar.noConsumerGroups') }}</span>
          </div>
          <div v-else-if="filteredGroups.length === 0 && groupSearchQuery" class="empty-list">
            <span>{{ t('sidebar.noMatch') }}</span>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="empty-state">
        <span class="empty-text">{{ t('sidebar.selectNav') }}</span>
      </div>
    </template>

    <!-- Resize Handle -->
    <div
      class="resize-handle"
      @mousedown="startResize"
      :class="{ 'resize-active': isResizing }"
    />
  </div>
</template>

<style scoped>
.second-sidebar {
  position: relative;
  height: 100%;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border-right: 1px solid var(--glass-border);
  overflow-y: auto;
  overflow-x: auto;
  flex-shrink: 0;
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
}

.second-sidebar:hover {
  scrollbar-color: var(--border) transparent;
}

.second-sidebar::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.second-sidebar::-webkit-scrollbar-track {
  background: transparent;
}

.second-sidebar::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 3px;
  transition: background 0.3s ease;
}

.second-sidebar:hover::-webkit-scrollbar-thumb {
  background: var(--border);
}

.second-sidebar::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

.second-sidebar::-webkit-scrollbar-corner {
  background: transparent;
}

.resize-handle {
  position: absolute;
  top: 0;
  right: 0;
  width: 4px;
  height: 100%;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s ease;
  z-index: 10;
}

.resize-handle:hover,
.resize-handle.resize-active {
  background: var(--accent);
}

.sidebar-section {
  padding: 12px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-title {
  font-size: 11px;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.search-row {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 8px;
}

.search-box {
  flex: 1;
}

.search-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.item-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-x: auto;
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
}

.item-list:hover {
  scrollbar-color: var(--border) transparent;
}

.item-list::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.item-list::-webkit-scrollbar-track {
  background: transparent;
}

.item-list::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 3px;
  transition: background 0.3s ease;
}

.item-list:hover::-webkit-scrollbar-thumb {
  background: var(--border);
}

.item-list::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

.list-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px;
  border-radius: var(--border-radius);
  cursor: pointer;
  color: var(--text-muted);
  transition: all 0.15s ease;
  font-weight: normal;
  white-space: nowrap;
  min-width: max-content;
  background: transparent;
}

.list-item:hover {
  background: var(--glass-bg-hover);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: var(--text-secondary);
}

.list-item.active {
  background: var(--glass-bg-active);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border-left: 2px solid var(--accent);
  color: var(--text-primary);
  font-weight: normal;
}

.item-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}

.item-name {
  font-size: 13px;
  white-space: nowrap;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.empty-list {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 24px;
  text-align: center;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 12px;
}

.empty-text {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 8px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.empty-hint {
  color: var(--text-muted);
  font-size: 12px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
}

.loading-spinner {
  color: var(--accent);
  animation: spin 1s linear infinite;
}

.loading-text {
  color: var(--text-muted);
  font-size: 13px;
}

.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: var(--border-radius);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.refresh-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spinning {
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

/* Light Mode */
:root[data-theme="light"] .second-sidebar {
  background: rgba(255, 255, 255, 0.6);
  border-right: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .resize-handle:hover,
:root[data-theme="light"] .resize-handle.resize-active {
  background: #3b82f6;
}

:root[data-theme="light"] .list-item {
  color: #64748b;
}

:root[data-theme="light"] .list-item:hover {
  background: rgba(255, 255, 255, 0.8);
  color: #475569;
}

:root[data-theme="light"] .list-item.active {
  background: rgba(59, 130, 246, 0.1);
  border-left: 2px solid #3b82f6;
  color: #1e293b;
}

:root[data-theme="light"] .item-icon {
  color: #64748b;
}

:root[data-theme="light"] .item-name {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .empty-text,
:root[data-theme="light"] .empty-hint {
  text-shadow: none;
}

:root[data-theme="light"] .empty-text {
  color: #475569;
}

:root[data-theme="light"] .empty-hint {
  color: #64748b;
}

:root[data-theme="light"] .refresh-btn {
  color: #64748b;
}

:root[data-theme="light"] .refresh-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.05);
  color: #475569;
}

:root[data-theme="light"] .search-icon {
  color: #64748b;
}
</style>
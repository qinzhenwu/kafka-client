<!-- src/App.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useTabStore } from '@/stores/tabs'
import { useThemeStore } from '@/stores/theme'
import { useClusterStore } from '@/stores/cluster'
import { useMessageStore } from '@/stores/message'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import { listenSwitchCluster } from '@/utils/window'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { check } from '@tauri-apps/plugin-updater'
import IconSidebar from '@/layouts/IconSidebar.vue'
import SecondSidebar from '@/layouts/SecondSidebar.vue'
import TabBar from '@/layouts/TabBar.vue'
import ClusterTabBar from '@/components/cluster/ClusterTabBar.vue'
import ClusterSwitch from '@/components/cluster/ClusterSwitch.vue'
import ClusterManager from '@/components/cluster/ClusterManager.vue'
import UpdateDialog from '@/components/common/UpdateDialog.vue'
import TopicDetailTab from '@/views/tabs/TopicDetailTab.vue'
import MessageTab from '@/views/tabs/MessageTab.vue'
import ConsumerGroupTab from '@/views/tabs/ConsumerGroupTab.vue'
import SendTestTab from '@/views/tabs/SendTestTab.vue'
import { BarChart3 } from 'lucide-vue-next'

// Initialize keyboard shortcuts
useKeyboardShortcuts()

const { t, locale } = useI18n()
const tabStore = useTabStore()
const themeStore = useThemeStore()
const clusterStore = useClusterStore()

const activeNav = ref<string>('topics')

const showClusterSwitch = ref(false)
const showClusterManager = ref(false)
const showUpdateDialog = ref(false)

// Tab content component mapping
const tabComponentMap: Record<string, any> = {
  'topic': TopicDetailTab,
  'message': MessageTab,
  'consumer-group': ConsumerGroupTab,
  'send-test': SendTestTab
}

const currentTabComponent = computed(() => {
  if (!tabStore.activeContentTab) return null
  return tabComponentMap[tabStore.activeContentTab.type]
})

const toggleLocale = () => {
  const newLocale = locale.value === 'zh-CN' ? 'en-US' : 'zh-CN'
  locale.value = newLocale
  localStorage.setItem('locale', newLocale)
}

const handleNavigate = (key: string) => {
  activeNav.value = key
}

const handleSelectTopic = (topicName: string) => {
  tabStore.openTab('topic', topicName, 'topic', { topicName })
}

const handleSelectConsumerGroup = (groupId: string) => {
  tabStore.openTab('consumer-group', groupId, 'consumerGroup', { groupId })
}

const handleOpenClusterSwitch = async () => {
  console.log('[App] Opening cluster switch')
  // 同步连接状态
  await clusterStore.syncConnectionStatus()
  showClusterSwitch.value = true
}

const handleCloseClusterSwitch = () => {
  console.log('[App] Closing cluster switch')
  showClusterSwitch.value = false
}

const handleOpenClusterManager = () => {
  console.log('[App] Opening cluster manager')
  showClusterSwitch.value = false
  showClusterManager.value = true
}

// Handle escape key to close popups
const handleEscapeKey = () => {
  if (showClusterManager.value) {
    showClusterManager.value = false
  } else if (showClusterSwitch.value) {
    showClusterSwitch.value = false
  }
}

// 清理函数引用
let unlistenSwitchCluster: (() => void) | null = null
let unlistenFocus: (() => void) | null = null
let unlistenClose: (() => void) | null = null

// 处理窗口关闭 - 断开当前窗口所有集群的连接
const handleWindowClose = async () => {
  const messageStore = useMessageStore()
  console.log('[App] Window closing, disconnecting clusters:', tabStore.clusterTabs.map(t => t.clientId))

  // 断开当前窗口所有集群的连接
  for (const tab of tabStore.clusterTabs) {
    // 1. 停止该集群的所有实时消费
    if (messageStore.isStreaming) {
      try {
        const messageTabs = tab.contentTabs.filter(t => t.type === 'message')
        for (const msgTab of messageTabs) {
          const topicName = msgTab.params.topicName
          if (messageStore.streamTopic === topicName) {
            console.log('[App] Stopping realtime consume for topic:', topicName)
            await invoke('stop_realtime_consume', { clientId: tab.clientId, topic: topicName })
          }
        }
      } catch (e) {
        console.error('[App] Failed to stop realtime consume:', e)
      }
    }

    // 2. 断开集群连接
    try {
      await invoke('disconnect_cluster', { clientId: tab.clientId })
      console.log('[App] Disconnected cluster:', tab.clientId)
    } catch (e) {
      console.error('[App] Failed to disconnect cluster:', tab.clientId, e)
    }
  }

  // 3. 清理消息 store 的监听器
  if (messageStore.isStreaming) {
    messageStore.isStreaming = false
    messageStore.streamTopic = null
    // unlisten 会在 store 中处理
  }
}

onMounted(async () => {
  window.addEventListener('escape-key', handleEscapeKey)
  themeStore.initTheme()

  const currentWindow = getCurrentWindow()

  // 监听窗口关闭事件
  unlistenClose = await currentWindow.onCloseRequested(async () => {
    console.log('[App] Close requested, disconnecting clusters')
    await handleWindowClose()
  })

  // 监听切换集群事件（用于从其他窗口切换回来时切换到对应集群）
  unlistenSwitchCluster = listenSwitchCluster((clientId: string) => {
    console.log('[App] Received switch-cluster-tab event for:', clientId)
    const tab = tabStore.clusterTabs.find(t => t.clientId === clientId)
    if (tab) {
      tabStore.switchClusterTab(tab.id)
    }
  })

  // 监听窗口焦点事件，同步连接状态
  unlistenFocus = await currentWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      console.log('[App] Window focused, syncing connection status')
      clusterStore.syncConnectionStatus()
    }
  })

  // Sync connection status from backend first
  await clusterStore.syncConnectionStatus()

  // 检查是否为单集群窗口
  const urlParams = new URLSearchParams(window.location.search)
  const singleCluster = urlParams.get('singleCluster')

  if (singleCluster === 'true') {
    const clientId = urlParams.get('clientId')
    const clusterId = urlParams.get('clusterId')
    const clusterName = urlParams.get('clusterName')

    if (clientId && clusterId && clusterName) {
      const decodedName = decodeURIComponent(clusterName)

      // 初始化单集群 Tab
      tabStore.initSingleClusterTab({
        clusterId,
        clusterName: decodedName,
        clientId,
        connected: true
      })

      // 查找并更新集群连接状态
      const existingCluster = clusterStore.clusters.find(c => c.id === clusterId)
      if (existingCluster) {
        existingCluster.connected = true
        existingCluster.clientId = clientId
      }

      // Fetch cluster info
      await clusterStore.fetchActiveClusterInfo(clientId)
    }
  }

  // Check for updates on startup (silent check)
  try {
    const update = await check()
    if (update) {
      console.log('[App] Update available:', update.version)
      showUpdateDialog.value = true
    }
  } catch (e) {
    console.log('[App] Update check failed (this is normal in dev mode):', e)
  }
})

onUnmounted(() => {
  window.removeEventListener('escape-key', handleEscapeKey)
  if (unlistenSwitchCluster) {
    unlistenSwitchCluster()
  }
  if (unlistenFocus) {
    unlistenFocus()
  }
  if (unlistenClose) {
    unlistenClose()
  }
})

// Initialize theme
themeStore.initTheme()
</script>

<template>
  <n-config-provider :theme="themeStore.isDark ? darkTheme : null">
    <n-message-provider>
      <n-dialog-provider>
        <div class="app-layout">
          <!-- Left Icon Sidebar -->
          <IconSidebar
            :active-nav="activeNav"
            @navigate="handleNavigate"
            @open-cluster-switch="handleOpenClusterSwitch"
          />

          <!-- Secondary Sidebar -->
          <SecondSidebar
            :active-nav="activeNav"
            @select-topic="handleSelectTopic"
            @select-consumer-group="handleSelectConsumerGroup"
          />

          <!-- Main Content Area -->
          <div class="main-area">
            <!-- Cluster Tab Bar -->
            <ClusterTabBar v-if="tabStore.clusterTabs.length > 0" />

            <!-- Tab Bar -->
            <TabBar
              @toggle-locale="toggleLocale"
            />

            <!-- Tab Content -->
            <div class="tab-content">
              <template v-if="tabStore.activeContentTab">
                <keep-alive>
                  <component
                    :is="currentTabComponent"
                    :key="tabStore.activeContentTab.id"
                    :tab="tabStore.activeContentTab"
                  />
                </keep-alive>
              </template>
              <template v-else>
                <div class="welcome-state">
                  <BarChart3 :size="48" :stroke-width="1" class="welcome-icon" />
                  <span class="welcome-text">{{ t('welcome.selectItem') }}</span>
                </div>
              </template>
            </div>
          </div>

          <!-- Cluster Switch Popup -->
          <div v-if="showClusterSwitch" class="cluster-popup-overlay" @click="handleCloseClusterSwitch">
            <div class="cluster-popup" @click.stop>
              <ClusterSwitch
                @close="handleCloseClusterSwitch"
                @open-manager="handleOpenClusterManager"
              />
            </div>
          </div>

          <!-- Cluster Manager Modal -->
          <ClusterManager
            v-if="showClusterManager"
            @close="showClusterManager = false"
          />

          <!-- Update Dialog -->
          <UpdateDialog v-model:show="showUpdateDialog" />
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--bg-primary);
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tab-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.welcome-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
}

.welcome-icon {
  color: var(--text-muted);
  margin-bottom: 16px;
}

.welcome-text {
  font-size: 14px;
}

.cluster-popup-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
}

.cluster-popup {
  position: absolute;
  left: calc(var(--icon-sidebar-width) + 8px);
  bottom: 16px;
}
</style>
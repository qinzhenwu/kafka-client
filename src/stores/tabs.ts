// src/stores/tabs.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { IconName } from '@/config/icons'

export type TabType = 'topic' | 'message' | 'consumer-group' | 'send-test'

export interface Tab {
  id: string
  type: TabType
  title: string
  icon: IconName
  params: Record<string, any>
}

// 集群 Tab 结构
export interface ClusterTab {
  id: string              // cluster-{clientId}
  clusterId: string       // 集群配置 ID
  clusterName: string     // 集群名称
  clientId: string        // Kafka 连接 ID
  connected: boolean      // 连接状态
  contentTabs: Tab[]      // 该集群下的内容 Tab
  activeContentTabId: string | null  // 当前激活的内容 Tab ID
}

export const useTabStore = defineStore('tabs', () => {
  // 新状态：集群 Tab 列表
  const clusterTabs = ref<ClusterTab[]>([])
  const activeClusterTabId = ref<string | null>(null)

  // Getters
  const activeClusterTab = computed(() =>
    clusterTabs.value.find(t => t.id === activeClusterTabId.value) || null
  )

  const activeClientId = computed(() => activeClusterTab.value?.clientId || null)

  const activeContentTabs = computed(() => activeClusterTab.value?.contentTabs || [])

  const activeContentTabId = computed(() => activeClusterTab.value?.activeContentTabId || null)

  const activeContentTab = computed(() => {
    const cluster = activeClusterTab.value
    if (!cluster) return null
    return cluster.contentTabs.find(t => t.id === cluster.activeContentTabId) || null
  })

  // Backward compatibility aliases (deprecated - use activeContentTabs/activeContentTabId/activeContentTab instead)
  const tabs = computed(() => activeContentTabs.value)
  const activeTabId = computed(() => activeContentTabId.value)
  const activeTab = computed(() => activeContentTab.value)

  // 生成集群 Tab ID
  const generateClusterTabId = (clientId: string): string => {
    return `cluster-${clientId}`
  }

  // 添加集群 Tab
  const addClusterTab = (cluster: Omit<ClusterTab, 'id' | 'contentTabs' | 'activeContentTabId'>): string => {
    const id = generateClusterTabId(cluster.clientId)
    const existingTab = clusterTabs.value.find(t => t.id === id)

    console.log('[TabStore] addClusterTab:', { id, existingTab: !!existingTab, cluster })

    if (existingTab) {
      // 已存在，切换到该 Tab
      activeClusterTabId.value = id
      console.log('[TabStore] Switched to existing tab, activeClusterTabId:', activeClusterTabId.value)
      return id
    }

    const newTab: ClusterTab = {
      ...cluster,
      id,
      contentTabs: [],
      activeContentTabId: null
    }
    clusterTabs.value.push(newTab)
    activeClusterTabId.value = id
    console.log('[TabStore] Added new tab, clusterTabs:', clusterTabs.value.length, 'activeClusterTabId:', activeClusterTabId.value, 'activeClientId:', activeClientId.value)
    return id
  }

  // 移除集群 Tab
  const removeClusterTab = (tabId: string): ClusterTab | null => {
    const index = clusterTabs.value.findIndex(t => t.id === tabId)
    if (index === -1) return null

    const removed = clusterTabs.value.splice(index, 1)[0]

    if (activeClusterTabId.value === tabId) {
      if (clusterTabs.value.length > 0) {
        const newIndex = Math.min(index, clusterTabs.value.length - 1)
        activeClusterTabId.value = clusterTabs.value[newIndex].id
      } else {
        activeClusterTabId.value = null
      }
    }

    return removed
  }

  // 切换集群 Tab
  const switchClusterTab = (tabId: string): void => {
    console.log('[TabStore] switchClusterTab:', tabId)
    if (clusterTabs.value.find(t => t.id === tabId)) {
      activeClusterTabId.value = tabId
      console.log('[TabStore] Switched, activeClusterTabId:', activeClusterTabId.value, 'activeClientId:', activeClientId.value)
    }
  }

  // 更新集群连接状态
  const updateClusterConnection = (tabId: string, connected: boolean): void => {
    const tab = clusterTabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.connected = connected
    }
  }

  // 初始化单集群 Tab（用于新窗口）
  const initSingleClusterTab = (cluster: Omit<ClusterTab, 'id' | 'contentTabs' | 'activeContentTabId'>): void => {
    clusterTabs.value = [{
      ...cluster,
      id: generateClusterTabId(cluster.clientId),
      contentTabs: [],
      activeContentTabId: null
    }]
    activeClusterTabId.value = generateClusterTabId(cluster.clientId)
  }

  // 内容 Tab Actions
  const generateTabId = (type: TabType, resourceId: string): string => {
    return `${type}-${resourceId}`
  }

  const openTab = (type: TabType, title: string, icon: IconName, params: Record<string, any>): string => {
    const cluster = activeClusterTab.value
    if (!cluster) return ''

    const resourceId = params.topicName || params.groupId || params.clusterId || Date.now().toString()
    const id = generateTabId(type, resourceId)

    const existingTab = cluster.contentTabs.find(t => t.id === id)
    if (existingTab) {
      cluster.activeContentTabId = id
      return id
    }

    const tab: Tab = { id, type, title, icon, params }
    cluster.contentTabs.push(tab)
    cluster.activeContentTabId = id
    return id
  }

  const closeTab = (tabId: string): void => {
    const cluster = activeClusterTab.value
    if (!cluster) return

    const index = cluster.contentTabs.findIndex(t => t.id === tabId)
    if (index === -1) return

    cluster.contentTabs.splice(index, 1)

    if (cluster.activeContentTabId === tabId) {
      if (cluster.contentTabs.length > 0) {
        const newIndex = Math.min(index, cluster.contentTabs.length - 1)
        cluster.activeContentTabId = cluster.contentTabs[newIndex].id
      } else {
        cluster.activeContentTabId = null
      }
    }
  }

  const closeAllTabs = (): void => {
    const cluster = activeClusterTab.value
    if (cluster) {
      cluster.contentTabs = []
      cluster.activeContentTabId = null
    }
  }

  const setActiveTab = (tabId: string): void => {
    const cluster = activeClusterTab.value
    if (cluster && cluster.contentTabs.find(t => t.id === tabId)) {
      cluster.activeContentTabId = tabId
    }
  }

  const closeOtherTabs = (tabId: string): void => {
    const cluster = activeClusterTab.value
    if (cluster) {
      cluster.contentTabs = cluster.contentTabs.filter(t => t.id === tabId)
      cluster.activeContentTabId = tabId
    }
  }

  const closeRightTabs = (tabId: string): void => {
    const cluster = activeClusterTab.value
    if (!cluster) return
    const index = cluster.contentTabs.findIndex(t => t.id === tabId)
    if (index === -1) return

    // Check if active tab is among those being closed
    const activeIndex = cluster.contentTabs.findIndex(t => t.id === cluster.activeContentTabId)
    if (activeIndex > index) {
      // Active tab is being closed, set active to the anchor tab
      cluster.activeContentTabId = tabId
    }

    cluster.contentTabs = cluster.contentTabs.slice(0, index + 1)
  }

  return {
    // State
    clusterTabs,
    activeClusterTabId,

    // Getters
    activeClusterTab,
    activeClientId,
    activeContentTabs,
    activeContentTabId,
    activeContentTab,

    // Backward compatibility aliases (deprecated)
    tabs,
    activeTabId,
    activeTab,

    // Cluster Tab Actions
    addClusterTab,
    removeClusterTab,
    switchClusterTab,
    updateClusterConnection,
    initSingleClusterTab,

    // Content Tab Actions
    openTab,
    closeTab,
    closeAllTabs,
    setActiveTab,
    closeOtherTabs,
    closeRightTabs
  }
})
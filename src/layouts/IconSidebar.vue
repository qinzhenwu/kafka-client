<!-- src/layouts/IconSidebar.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useClusterStore } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import IconButton from '@/components/common/IconButton.vue'
import { Server } from 'lucide-vue-next'
import type { IconName } from '@/config/icons'

interface NavItem {
  key: string
  icon: IconName
  tooltipKey: string
}

const { t } = useI18n()
const clusterStore = useClusterStore()
const tabStore = useTabStore()

const { activeNav } = defineProps<{
  activeNav: string
}>()

const emit = defineEmits<{
  navigate: [key: string]
  openClusterSwitch: []
}>()

const navItems: NavItem[] = [
  { key: 'topics', icon: 'topic', tooltipKey: 'menu.topic' },
  { key: 'consumer-groups', icon: 'consumerGroup', tooltipKey: 'menu.consumerGroup' }
]

// 通过 clusterId 查找当前集群（而不是 clientId）
const currentCluster = computed(() => {
  const activeTab = tabStore.activeClusterTab
  if (!activeTab) return null
  return clusterStore.clusters.find(c => c.id === activeTab.clusterId)
})

const clusterStatus = computed(() => {
  // 优先使用 tab 的 connected 状态
  const activeTab = tabStore.activeClusterTab
  if (activeTab) {
    return activeTab.connected ? 'connected' : 'disconnected'
  }
  return currentCluster.value?.connected ? 'connected' : 'disconnected'
})

const clusterTooltip = computed(() => {
  const activeTab = tabStore.activeClusterTab
  if (activeTab) {
    const cluster = clusterStore.clusters.find(c => c.id === activeTab.clusterId)
    const name = cluster?.name || activeTab.clusterName
    const bootstrap = cluster?.config?.bootstrap_servers || ''
    const status = activeTab.connected ? '● ' + t('cluster.connected') : '○ ' + t('cluster.disconnected')
    return `${name}\n${bootstrap}\n${status}`
  }
  return t('empty.noCluster')
})

const handleNavClick = (key: string) => {
  emit('navigate', key)
}
</script>

<template>
  <div class="icon-sidebar">
    <!-- Navigation icons -->
    <div class="nav-icons">
      <IconButton
        v-for="item in navItems"
        :key="item.key"
        :icon="item.icon"
        :active="activeNav === item.key"
        :tooltip="t(item.tooltipKey)"
        size="medium"
        @click="handleNavClick(item.key)"
      />
    </div>

    <!-- Cluster switcher at bottom -->
    <div class="cluster-switcher">
      <div
        class="cluster-icon"
        :title="clusterTooltip"
        @click="emit('openClusterSwitch')"
      >
        <Server :size="22" :stroke-width="1.5" class="cluster-emoji" />
        <span v-if="currentCluster" class="cluster-name">{{ currentCluster.name.slice(0, 2) }}</span>
        <span
          class="status-dot"
          :class="clusterStatus"
        ></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.icon-sidebar {
  width: var(--icon-sidebar-width);
  height: 100%;
  background: var(--bg-tertiary);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  border-right: 1px solid var(--border);
}

.nav-icons {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 0 6px;
}

.cluster-switcher {
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.cluster-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 4px;
  border-radius: var(--border-radius);
  cursor: pointer;
  position: relative;
  border: 1px solid var(--border);
  background: var(--bg-secondary);
}

.cluster-icon:hover {
  background: var(--bg-hover);
}

.cluster-emoji {
  color: var(--accent);
}

.cluster-name {
  font-size: 9px;
  color: var(--text-muted);
  margin-top: 2px;
  max-width: 40px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-dot {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot.connected {
  background: var(--success);
}

.status-dot.disconnected {
  background: var(--error);
}
</style>
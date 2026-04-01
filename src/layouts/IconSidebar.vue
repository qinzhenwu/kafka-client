<!-- src/layouts/IconSidebar.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useClusterStore } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import IconButton from '@/components/common/IconButton.vue'
import { Server, Settings } from 'lucide-vue-next'
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
  openSettings: []
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
    <div class="bottom-section">
      <!-- Settings button -->
      <div class="settings-btn" :title="t('common.settings')" @click="emit('openSettings')">
        <Settings :size="22" :stroke-width="1.5" class="settings-icon" />
      </div>

      <!-- Cluster switcher -->
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
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  border-right: 1px solid var(--glass-border);
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
  border-top: 1px solid var(--glass-border);
}

.bottom-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--glass-border);
}

.settings-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 4px;
  border-radius: var(--border-radius);
  cursor: pointer;
  color: var(--text-muted);
  transition: all 0.15s ease;
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.settings-btn:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}

.settings-icon {
  color: var(--accent);
}

.cluster-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 4px;
  border-radius: var(--border-radius);
  cursor: pointer;
  position: relative;
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.cluster-icon:hover {
  background: var(--glass-bg-hover);
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
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
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

/* Light Mode */
:root[data-theme="light"] .icon-sidebar {
  background: rgba(255, 255, 255, 0.6);
  border-right: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .bottom-section {
  border-top: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .settings-btn {
  color: #64748b;
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: rgba(255, 255, 255, 0.4);
}

:root[data-theme="light"] .settings-btn:hover {
  background: rgba(255, 255, 255, 0.7);
  color: #1e293b;
}

:root[data-theme="light"] .cluster-icon {
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: rgba(255, 255, 255, 0.4);
}

:root[data-theme="light"] .cluster-icon:hover {
  background: rgba(255, 255, 255, 0.7);
}

:root[data-theme="light"] .cluster-name {
  color: #64748b;
  text-shadow: none;
}
</style>
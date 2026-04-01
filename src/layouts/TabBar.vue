<!-- src/layouts/TabBar.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTabStore, type Tab } from '@/stores/tabs'
import { useThemeStore } from '@/stores/theme'
import { icons } from '@/config/icons'
import { X, Sun, Moon, Monitor, ChevronLeft, ChevronRight } from 'lucide-vue-next'

const { t, locale } = useI18n()
const tabStore = useTabStore()
const themeStore = useThemeStore()

const tabsContainerRef = ref<HTMLElement | null>(null)

const showLeftArrow = ref(false)
const showRightArrow = ref(false)

const emit = defineEmits<{
  toggleLocale: []
}>()

const currentModeIcon = computed(() => {
  switch (themeStore.mode) {
    case 'light': return Sun
    case 'dark': return Moon
    case 'system': return Monitor
  }
})

const getTabTitle = (tab: Tab): string => {
  return tab.title
}

const handleCloseTab = (tabId: string, event: Event) => {
  event.stopPropagation()
  tabStore.closeTab(tabId)
}

const handleToggleTheme = () => {
  themeStore.toggleMode()
}

const handleToggleLocale = () => {
  emit('toggleLocale')
}

// Check if arrows should be shown
const checkArrows = () => {
  if (!tabsContainerRef.value) return
  const container = tabsContainerRef.value
  showLeftArrow.value = container.scrollLeft > 0
  showRightArrow.value = container.scrollLeft < container.scrollWidth - container.clientWidth - 1
}

// Scroll tabs
const scrollTabs = (direction: 'left' | 'right') => {
  if (!tabsContainerRef.value) return
  const container = tabsContainerRef.value
  const scrollAmount = 150
  container.scrollBy({
    left: direction === 'left' ? -scrollAmount : scrollAmount,
    behavior: 'smooth'
  })
}

// Scroll to active tab
const scrollToActiveTab = () => {
  if (!tabsContainerRef.value) return
  const container = tabsContainerRef.value
  const activeTabEl = container.querySelector('.tab.active') as HTMLElement
  if (activeTabEl) {
    const containerRect = container.getBoundingClientRect()
    const tabRect = activeTabEl.getBoundingClientRect()
    if (tabRect.left < containerRect.left || tabRect.right > containerRect.right) {
      activeTabEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' })
    }
  }
}

// Update arrows when tabs change
watch(() => tabStore.activeContentTabs.length, () => {
  nextTick(() => {
    checkArrows()
    scrollToActiveTab()
  })
})

// Update arrows when active tab changes
watch(() => tabStore.activeContentTabId, () => {
  nextTick(scrollToActiveTab)
})

onMounted(() => {
  if (tabsContainerRef.value) {
    tabsContainerRef.value.addEventListener('scroll', checkArrows)
  }
  checkArrows()
})

onUnmounted(() => {
  if (tabsContainerRef.value) {
    tabsContainerRef.value.removeEventListener('scroll', checkArrows)
  }
})
</script>

<template>
  <div class="tab-bar">
    <!-- Left Arrow -->
    <button
      v-if="showLeftArrow"
      class="tab-arrow left"
      @click="scrollTabs('left')"
    >
      <ChevronLeft :size="16" />
    </button>

    <div class="tabs-container" ref="tabsContainerRef">
      <div
        v-for="tab in tabStore.activeContentTabs"
        :key="tab.id"
        class="tab"
        :class="{ active: tabStore.activeContentTabId === tab.id }"
        @click="tabStore.setActiveTab(tab.id)"
      >
        <component :is="icons[tab.icon]" :size="14" :stroke-width="1.5" class="tab-icon" />
        <span class="tab-title" :title="getTabTitle(tab)">{{ getTabTitle(tab) }}</span>
        <X :size="12" class="tab-close" @click="handleCloseTab(tab.id, $event)" />
      </div>
    </div>

    <!-- Right Arrow -->
    <button
      v-if="showRightArrow"
      class="tab-arrow right"
      @click="scrollTabs('right')"
    >
      <ChevronRight :size="16" />
    </button>

    <div class="tab-actions">
      <button
        class="theme-mode-toggle"
        :title="t('tooltip.toggleTheme')"
        @click="handleToggleTheme"
      >
        <component :is="currentModeIcon" :size="14" :stroke-width="1.5" />
      </button>
      <button
        class="locale-btn"
        :title="t('tooltip.switchLanguage')"
        @click="handleToggleLocale"
      >
        {{ locale === 'zh-CN' ? '中' : 'EN' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.tab-bar {
  position: relative;
  display: flex;
  align-items: center;
  height: var(--tab-bar-height);
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border-bottom: 1px solid var(--glass-border);
  z-index: 100;
}

.tabs-container {
  flex: 1;
  display: flex;
  overflow-x: auto;
  gap: 2px;
  padding: 0 8px;
  scrollbar-width: none;
}

.tabs-container::-webkit-scrollbar {
  display: none;
}

.tab-arrow {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 100%;
  background: var(--glass-bg);
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-arrow:hover {
  background: var(--glass-bg-hover);
  color: var(--text-secondary);
}

.tab-arrow.left {
  border-right: 1px solid var(--glass-border);
}

.tab-arrow.right {
  border-left: 1px solid var(--glass-border);
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--glass-bg);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  border-radius: var(--border-radius) var(--border-radius) 0 0;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.tab:hover {
  background: var(--glass-bg-hover);
  color: var(--text-secondary);
}

.tab.active {
  background: var(--glass-bg-active);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent);
}

.tab-icon {
  flex-shrink: 0;
}

.tab-title {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.tab-close {
  color: var(--text-muted);
  padding: 2px;
  border-radius: 3px;
  transition: all 0.15s ease;
  cursor: pointer;
}

.tab-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tab-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 8px;
  border-left: 1px solid var(--glass-border);
}

.locale-btn {
  padding: 4px;
  background: transparent;
  border: none;
  border-radius: var(--border-radius);
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.locale-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.theme-mode-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: var(--border-radius);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-mode-toggle:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

/* Light Mode */
:root[data-theme="light"] .tab-bar {
  background: rgba(255, 255, 255, 0.6);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .tab-arrow {
  background: rgba(255, 255, 255, 0.6);
  color: #64748b;
}

:root[data-theme="light"] .tab-arrow:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #475569;
}

:root[data-theme="light"] .tab-arrow.left {
  border-right: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .tab-arrow.right {
  border-left: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .tab {
  background: rgba(255, 255, 255, 0.4);
  color: #64748b;
}

:root[data-theme="light"] .tab:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #475569;
}

:root[data-theme="light"] .tab.active {
  background: rgba(59, 130, 246, 0.1);
  color: #1e293b;
  border-bottom: 2px solid #3b82f6;
}

:root[data-theme="light"] .tab-title {
  color: #1e293b;
  text-shadow: none;
}

:root[data-theme="light"] .tab-close {
  color: #64748b;
}

:root[data-theme="light"] .tab-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1e293b;
}

:root[data-theme="light"] .tab-actions {
  border-left: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .locale-btn {
  color: #64748b;
  text-shadow: none;
}

:root[data-theme="light"] .locale-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #475569;
}

:root[data-theme="light"] .theme-mode-toggle {
  color: #64748b;
}

:root[data-theme="light"] .theme-mode-toggle:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #475569;
}
</style>
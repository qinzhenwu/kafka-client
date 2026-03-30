<!-- src/layouts/TabBar.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTabStore, type Tab } from '@/stores/tabs'
import { useThemeStore, type ColorScheme, type ThemeMode } from '@/stores/theme'
import { icons } from '@/config/icons'
import { X, Palette, Sun, Moon, Monitor, ChevronLeft, ChevronRight } from 'lucide-vue-next'

const { t, locale } = useI18n()
const tabStore = useTabStore()
const themeStore = useThemeStore()

const showThemeMenu = ref(false)
const themeMenuRef = ref<HTMLElement | null>(null)
const tabsContainerRef = ref<HTMLElement | null>(null)

const showLeftArrow = ref(false)
const showRightArrow = ref(false)

const emit = defineEmits<{
  toggleLocale: []
}>()

const colorSchemes: { value: ColorScheme; label: string }[] = [
  { value: 'midnight-blue', label: 'Midnight Blue' },
  { value: 'slate-gray', label: 'Slate Gray' },
]

const themeModes: { value: ThemeMode; label: string; icon: any }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'System', icon: Monitor },
]

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

const handleSelectColorScheme = (scheme: ColorScheme) => {
  themeStore.setColorScheme(scheme)
}

const handleSelectMode = (mode: ThemeMode) => {
  themeStore.setMode(mode)
}

const handleClickOutside = (event: MouseEvent) => {
  if (themeMenuRef.value && !themeMenuRef.value.contains(event.target as Node)) {
    showThemeMenu.value = false
  }
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
  document.addEventListener('click', handleClickOutside)
  if (tabsContainerRef.value) {
    tabsContainerRef.value.addEventListener('scroll', checkArrows)
  }
  checkArrows()
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
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
        :title="getTabTitle(tab)"
        @click="tabStore.setActiveTab(tab.id)"
      >
        <component :is="icons[tab.icon]" :size="14" :stroke-width="1.5" class="tab-icon" />
        <span class="tab-title">{{ getTabTitle(tab) }}</span>
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
      <div class="theme-selector" ref="themeMenuRef">
        <button
          class="theme-scheme-btn"
          :title="t('tooltip.selectTheme')"
          @click="showThemeMenu = !showThemeMenu"
        >
          <Palette :size="14" :stroke-width="1.5" />
        </button>
        <div v-if="showThemeMenu" class="theme-menu">
          <!-- Mode Selection -->
          <div class="theme-section">
            <div class="theme-section-title">{{ t('theme.mode') }}</div>
            <div class="theme-modes">
              <button
                v-for="mode in themeModes"
                :key="mode.value"
                class="theme-mode-btn"
                :class="{ active: themeStore.mode === mode.value }"
                @click="handleSelectMode(mode.value)"
              >
                <component :is="mode.icon" :size="14" />
                <span>{{ mode.label }}</span>
              </button>
            </div>
          </div>
          <!-- Color Scheme Selection -->
          <div class="theme-section">
            <div class="theme-section-title">{{ t('theme.colorScheme') }}</div>
            <div
              v-for="scheme in colorSchemes"
              :key="scheme.value"
              class="theme-menu-item"
              :class="{ active: themeStore.colorScheme === scheme.value }"
              @click="handleSelectColorScheme(scheme.value)"
            >
              <span class="color-preview" :class="scheme.value"></span>
              <span>{{ scheme.label }}</span>
            </div>
          </div>
        </div>
      </div>
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
  display: flex;
  align-items: center;
  height: var(--tab-bar-height);
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border);
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
  background: var(--bg-tertiary);
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-arrow:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.tab-arrow.left {
  border-right: 1px solid var(--border);
}

.tab-arrow.right {
  border-left: 1px solid var(--border);
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  border-radius: var(--border-radius) var(--border-radius) 0 0;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.tab:hover {
  background: var(--bg-secondary);
  color: var(--text-secondary);
}

.tab.active {
  background: var(--bg-primary);
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
  border-left: 1px solid var(--border);
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
}

.locale-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.theme-selector {
  position: relative;
}

.theme-scheme-btn {
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

.theme-scheme-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.theme-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--border-radius);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  z-index: 100;
  overflow: hidden;
}

.theme-section {
  padding: 8px 0;
}

.theme-section:not(:last-child) {
  border-bottom: 1px solid var(--border);
}

.theme-section-title {
  padding: 4px 14px;
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.theme-modes {
  display: flex;
  gap: 4px;
  padding: 0 10px;
}

.theme-mode-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 4px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--border-radius);
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-mode-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-color: var(--text-muted);
}

.theme-mode-btn.active {
  background: var(--accent-bg);
  color: var(--accent);
  border-color: var(--accent);
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

.theme-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-menu-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.theme-menu-item.active {
  background: var(--accent-bg);
  color: var(--accent);
}

.color-preview {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  border: 1px solid var(--border);
}

.color-preview.midnight-blue {
  background: linear-gradient(135deg, #0f172a 50%, #3b82f6 50%);
}

.color-preview.slate-gray {
  background: linear-gradient(135deg, #18181b 50%, #a1a1aa 50%);
}
</style>
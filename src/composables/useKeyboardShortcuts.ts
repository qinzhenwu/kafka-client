// src/composables/useKeyboardShortcuts.ts
import { onMounted, onUnmounted } from 'vue'
import { useTabStore } from '@/stores/tabs'

// Custom event for escape key
export const escapeKeyEvent = new CustomEvent('escape-key')

export function useKeyboardShortcuts() {
  const tabStore = useTabStore()

  const handleKeydown = (e: KeyboardEvent) => {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
    const modifier = isMac ? e.metaKey : e.ctrlKey

    // Escape: Close popups/panels
    if (e.key === 'Escape') {
      window.dispatchEvent(new CustomEvent('escape-key'))
    }

    // Ctrl/Cmd + Tab: Next/Previous tab
    if (modifier && e.key === 'Tab') {
      e.preventDefault()
      if (tabStore.tabs.length === 0) return

      const currentIndex = tabStore.tabs.findIndex(t => t.id === tabStore.activeTabId)
      let nextIndex: number

      if (e.shiftKey) {
        // Previous tab
        nextIndex = currentIndex > 0 ? currentIndex - 1 : tabStore.tabs.length - 1
      } else {
        // Next tab
        nextIndex = currentIndex < tabStore.tabs.length - 1 ? currentIndex + 1 : 0
      }

      tabStore.setActiveTab(tabStore.tabs[nextIndex].id)
    }

    // Ctrl/Cmd + W: Close current tab
    if (modifier && e.key === 'w') {
      e.preventDefault()
      if (tabStore.activeTabId) {
        tabStore.closeTab(tabStore.activeTabId)
      }
    }

    // Ctrl/Cmd + 1-9: Switch to tab N
    if (modifier && e.key >= '1' && e.key <= '9') {
      e.preventDefault()
      const index = parseInt(e.key) - 1
      if (tabStore.tabs[index]) {
        tabStore.setActiveTab(tabStore.tabs[index].id)
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })
}
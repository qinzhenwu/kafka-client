// src/composables/useClusterDrag.ts
import { ref, type Ref, onMounted, onUnmounted } from 'vue'
import type { ClusterTab } from '@/stores/tabs'
import { createClusterWindow } from '@/utils/window'

export interface UseClusterDragOptions {
  onDragOut?: (tab: ClusterTab) => void
}

export interface UseClusterDragReturn {
  handleDragStart: (tabId: string, event: DragEvent) => void
  handleDragEnd: (event: DragEvent) => Promise<void>
  isDragging: Ref<boolean>
  draggingTabId: Ref<string | null>
}

export function useClusterDrag(
  clusterTabs: Ref<ClusterTab[]>,
  options?: UseClusterDragOptions
): UseClusterDragReturn {
  const isDragging = ref(false)
  const draggingTabId = ref<string | null>(null)

  // Track if a drop happened within the window
  let dropHappened = false

  // Handle drop event in the window
  const handleDrop = (event: DragEvent) => {
    console.log('[useClusterDrag] Drop event fired')
    dropHappened = true
    // Prevent default behavior
    event.preventDefault()
  }

  // Handle dragover to allow drop
  const handleDragOver = (event: DragEvent) => {
    event.preventDefault()
  }

  const handleDragStart = (tabId: string, event: DragEvent) => {
    isDragging.value = true
    draggingTabId.value = tabId
    dropHappened = false

    // Set drag data
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
      event.dataTransfer.setData('text/plain', tabId)
      // Set a drag image to make it clear we're dragging
      const dragImage = event.target as HTMLElement
      if (dragImage) {
        event.dataTransfer.setDragImage(dragImage, 0, 0)
      }
    }

    console.log('[useClusterDrag] Drag start:', { tabId })
  }

  const handleDragEnd = async (event: DragEvent) => {
    console.log('[useClusterDrag] Drag end:', {
      isDragging: isDragging.value,
      draggingTabId: draggingTabId.value,
      dropHappened,
      clientX: event.clientX,
      clientY: event.clientY,
      screenX: event.screenX,
      screenY: event.screenY
    })

    if (!isDragging.value || !draggingTabId.value) {
      reset()
      return
    }

    const tab = clusterTabs.value.find(t => t.id === draggingTabId.value)
    if (!tab) {
      console.log('[useClusterDrag] Tab not found')
      reset()
      return
    }

    // Check if the drag ended outside the window
    // If dropHappened is false and the mouse is outside the window bounds, create new window
    const isOutsideWindow = !dropHappened && (
      event.clientX < 0 ||
      event.clientY < 0 ||
      event.clientX > window.innerWidth ||
      event.clientY > window.innerHeight
    )

    console.log('[useClusterDrag] isOutsideWindow:', isOutsideWindow, {
      windowWidth: window.innerWidth,
      windowHeight: window.innerHeight
    })

    if (isOutsideWindow) {
      console.log('[useClusterDrag] Creating new window for tab:', tab.clusterName)
      try {
        const webviewWindow = await createClusterWindow(tab)
        console.log('[useClusterDrag] Window created:', webviewWindow.label)

        // Wait a bit for the window to initialize
        await new Promise(resolve => setTimeout(resolve, 100))

        // Remove the tab from current window
        options?.onDragOut?.(tab)
      } catch (e) {
        console.error('[useClusterDrag] Failed to create new window:', e)
      }
    }

    reset()
  }

  const reset = () => {
    isDragging.value = false
    draggingTabId.value = null
    dropHappened = false
  }

  // Add global event listeners
  onMounted(() => {
    document.addEventListener('drop', handleDrop)
    document.addEventListener('dragover', handleDragOver)
  })

  onUnmounted(() => {
    document.removeEventListener('drop', handleDrop)
    document.removeEventListener('dragover', handleDragOver)
  })

  return {
    handleDragStart,
    handleDragEnd,
    isDragging,
    draggingTabId
  }
}
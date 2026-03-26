// src/utils/window.ts
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { emit } from '@tauri-apps/api/event'
import type { ClusterTab } from '@/stores/tabs'

const SWITCH_CLUSTER_EVENT = 'switch-cluster-tab'

function getWindowUrl(params: URLSearchParams): string {
  // In development, use the dev server URL
  if (import.meta.env.DEV) {
    return `http://localhost:1420/?${params.toString()}`
  }
  // In production, use relative path
  return `index.html?${params.toString()}`
}

export async function createClusterWindow(clusterTab: ClusterTab): Promise<WebviewWindow> {
  const windowLabel = `cluster-${clusterTab.clientId}`

  console.log('[createClusterWindow] Creating window with label:', windowLabel)

  // 检查窗口是否已存在
  const existingWindow = await WebviewWindow.getByLabel(windowLabel)
  if (existingWindow) {
    console.log('[createClusterWindow] Window already exists, focusing')
    await existingWindow.setFocus()
    return existingWindow
  }

  const params = new URLSearchParams({
    clientId: clusterTab.clientId,
    clusterId: clusterTab.clusterId,
    clusterName: clusterTab.clusterName,
    singleCluster: 'true'
  })

  const url = getWindowUrl(params)
  console.log('[createClusterWindow] Window URL:', url)

  // 创建新窗口
  const webview = new WebviewWindow(windowLabel, {
    title: clusterTab.clusterName,
    width: 1280,
    height: 800,
    url
  })

  // Listen for errors
  webview.once('tauri://error', (e) => {
    console.error('[createClusterWindow] Window error:', e)
  })

  console.log('[createClusterWindow] Window created successfully')
  return webview
}

// 切换集群事件
export interface SwitchClusterEvent {
  clientId: string
}

// 监听切换集群事件
export function listenSwitchCluster(callback: (clientId: string) => void): () => void {
  let unlisten: (() => void) | null = null

  import('@tauri-apps/api/event').then(({ listen }) => {
    listen<SwitchClusterEvent>(SWITCH_CLUSTER_EVENT, (event) => {
      console.log('[listenSwitchCluster] Received event:', event.payload)
      callback(event.payload.clientId)
    }).then((fn) => {
      unlisten = fn
    })
  })

  return () => {
    if (unlisten) unlisten()
  }
}

// 发送切换集群事件
export async function emitSwitchCluster(clientId: string): Promise<void> {
  await emit(SWITCH_CLUSTER_EVENT, { clientId })
}

// 聚焦包含指定集群的窗口
// 返回值：'cluster-window' - 独立窗口, 'main-window' - 主窗口, null - 未找到
export async function focusClusterWindow(clientId: string): Promise<'cluster-window' | 'main-window' | null> {
  // 先尝试找独立窗口
  const clusterWindowLabel = `cluster-${clientId}`
  const clusterWindow = await WebviewWindow.getByLabel(clusterWindowLabel)
  if (clusterWindow) {
    console.log('[focusClusterWindow] Found cluster window:', clusterWindowLabel)
    await clusterWindow.setFocus()
    return 'cluster-window'
  }

  // 尝试聚焦主窗口（集群可能在主窗口中）
  const mainWindow = await WebviewWindow.getByLabel('main')
  if (mainWindow) {
    console.log('[focusClusterWindow] Found main window, focusing and emitting switch event')
    // 先发送切换事件，再聚焦
    await emitSwitchCluster(clientId)
    await mainWindow.setFocus()
    return 'main-window'
  }

  console.log('[focusClusterWindow] No window found for cluster:', clientId)
  return null
}
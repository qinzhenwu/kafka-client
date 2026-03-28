import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export interface UpdateInfo {
  version: string
  currentVersion: string
  date?: string
  body?: string
}

export function useUpdater() {
  const checking = ref(false)
  const updateAvailable = ref(false)
  const downloading = ref(false)
  const downloadProgress = ref(0)
  const updateInfo = ref<UpdateInfo | null>(null)
  const error = ref<string | null>(null)

  let update: Awaited<ReturnType<typeof check>> = null

  const checkForUpdate = async () => {
    checking.value = true
    error.value = null
    updateAvailable.value = false
    updateInfo.value = null

    try {
      // 使用 insecure 选项跳过签名验证（应用未签名）
      update = await check({
        dangerousInsecureTransport: true
      })

      if (update) {
        updateAvailable.value = true
        updateInfo.value = {
          version: update.version,
          currentVersion: update.currentVersion,
          date: update.date,
          body: update.body
        }
      }
    } catch (e) {
      error.value = String(e)
      console.error('Failed to check for updates:', e)
    } finally {
      checking.value = false
    }

    return updateAvailable.value
  }

  const downloadAndInstall = async () => {
    if (!update) return

    downloading.value = true
    downloadProgress.value = 0

    try {
      await update.downloadAndInstall((progress) => {
        switch (progress.event) {
          case 'Started':
            downloadProgress.value = 0
            break
          case 'Progress':
            // progress.data contains chunkLength
            break
          case 'Finished':
            downloadProgress.value = 100
            break
        }
      })

      // Relaunch the app after update
      await relaunch()
    } catch (e) {
      error.value = String(e)
      console.error('Failed to download and install update:', e)
    } finally {
      downloading.value = false
    }
  }

  return {
    checking,
    updateAvailable,
    downloading,
    downloadProgress,
    updateInfo,
    error,
    checkForUpdate,
    downloadAndInstall
  }
}
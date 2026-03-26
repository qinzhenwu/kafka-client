// src/utils/tauri.ts
// Utility functions for Tauri environment detection and safe API access

/**
 * Check if running in Tauri environment
 */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

/**
 * Safely get Tauri invoke function
 * Returns null if not in Tauri environment
 */
export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error('Not running in Tauri environment. Please use "npm run tauri dev" instead of "npm run dev"')
  }

  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd, args)
}
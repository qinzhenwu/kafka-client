import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useTabStore } from './tabs'

const CLUSTERS_STORAGE_KEY = 'kafka-clusters'

export interface ClusterConfig {
  id: string
  name: string
  bootstrap_servers: string
  security_protocol: 'PLAINTEXT' | 'SSL' | 'SASL_PLAINTEXT' | 'SASL_SSL'
  sasl_mechanism?: 'PLAIN' | 'SCRAM_SHA_256' | 'SCRAM_SHA_512' | 'GSSAPI'
  sasl_username?: string
  sasl_password?: string
  ssl_ca_location?: string
  ssl_certificate_location?: string
  ssl_key_location?: string
  ssl_key_password?: string
}

export interface BrokerInfo {
  id: number
  host: string
  port: number
  rack?: string
}

export interface ClusterInfo {
  cluster_id: string
  controller_id: number
  brokers: BrokerInfo[]
}

export interface ClusterItem {
  id: string
  name: string
  config: ClusterConfig
  connected: boolean
  clientId?: string
}

interface ClusterState {
  clusters: ClusterItem[]
  activeClusterInfo: ClusterInfo | null
  loading: boolean
  error: string | null
}

// Helper functions for localStorage persistence
interface StoredCluster {
  id: string
  name: string
  config: ClusterConfig
}

function loadClustersFromStorage(): StoredCluster[] {
  try {
    const stored = localStorage.getItem(CLUSTERS_STORAGE_KEY)
    return stored ? JSON.parse(stored) : []
  } catch {
    return []
  }
}

function saveClustersToStorage(clusters: StoredCluster[]) {
  localStorage.setItem(CLUSTERS_STORAGE_KEY, JSON.stringify(clusters))
}

function getStoredClustersAsItems(): ClusterItem[] {
  const stored = loadClustersFromStorage()
  return stored.map(s => ({
    id: s.id,
    name: s.name,
    config: s.config,
    connected: false
  }))
}

export const useClusterStore = defineStore('cluster', {
  state: (): ClusterState => ({
    clusters: getStoredClustersAsItems(),
    activeClusterInfo: null,
    loading: false,
    error: null,
  }),

  getters: {
    activeClusterId(): string | null {
      const tabStore = useTabStore()
      return tabStore.activeClientId
    }
  },

  actions: {
    async testClusterConnection(config: ClusterConfig): Promise<ClusterInfo> {
      this.loading = true
      this.error = null
      try {
        const info = await invoke<ClusterInfo>('test_cluster_connection', { config })
        return info
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async connectCluster(config: ClusterConfig) {
      this.loading = true
      this.error = null
      try {
        const client_id = await invoke<string>('connect_cluster', { config })

        const existingIndex = this.clusters.findIndex(c => c.id === config.id)
        if (existingIndex >= 0) {
          // Update all fields when reconnecting
          this.clusters[existingIndex].name = config.name
          this.clusters[existingIndex].config = config
          this.clusters[existingIndex].connected = true
          this.clusters[existingIndex].clientId = client_id
        } else {
          this.clusters.push({
            id: config.id,
            name: config.name,
            config,
            connected: true,
            clientId: client_id
          })
        }

        saveClustersToStorage(this.clusters.map(c => ({ id: c.id, name: c.name, config: c.config })))
        return client_id
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    // Update cluster config without reconnecting
    updateCluster(config: ClusterConfig) {
      const existingIndex = this.clusters.findIndex(c => c.id === config.id)
      if (existingIndex >= 0) {
        this.clusters[existingIndex].name = config.name
        this.clusters[existingIndex].config = config
        saveClustersToStorage(this.clusters.map(c => ({ id: c.id, name: c.name, config: c.config })))
      }
    },

    async disconnectCluster(clientId: string) {
      try {
        const cluster = this.clusters.find(c => c.clientId === clientId || c.id === clientId)
        if (cluster) {
          const clusterClientId = cluster.clientId || clientId
          await invoke('disconnect_cluster', { clientId: clusterClientId })
          cluster.connected = false
          cluster.clientId = undefined
        }
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      }
    },

    async deleteCluster(clusterId: string) {
      const cluster = this.clusters.find(c => c.id === clusterId)
      if (cluster?.connected && cluster.clientId) {
        try {
          await invoke('disconnect_cluster', { clientId: cluster.clientId })
        } catch {
          // Ignore disconnect errors when deleting
        }
      }
      this.clusters = this.clusters.filter(c => c.id !== clusterId)
      saveClustersToStorage(this.clusters.map(c => ({ id: c.id, name: c.name, config: c.config })))
    },

    async fetchClusterInfo(clientId: string): Promise<ClusterInfo> {
      try {
        const info = await invoke<ClusterInfo>('get_cluster_info', { clientId })
        return info
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      }
    },

    // Fetch and store cluster info for the active cluster
    async fetchActiveClusterInfo(clientId: string): Promise<void> {
      try {
        const info = await invoke<ClusterInfo>('get_cluster_info', { clientId })
        this.activeClusterInfo = info
      } catch (e: unknown) {
        this.error = String(e)
        this.activeClusterInfo = null
      }
    },

    // Clear active cluster info when no cluster is active
    clearActiveClusterInfo(): void {
      this.activeClusterInfo = null
    },

    // Sync connection status from backend
    async syncConnectionStatus(): Promise<void> {
      try {
        const connectedClientIds = await invoke<string[]>('list_connected_clusters')
        console.log('[ClusterStore] Connected client IDs from backend:', connectedClientIds)
        console.log('[ClusterStore] Current clusters state:', this.clusters.map(c => ({ id: c.id, name: c.name, connected: c.connected })))

        // Update connection status for all clusters
        for (const cluster of this.clusters) {
          const wasConnected = cluster.connected
          const isConnected = connectedClientIds.includes(cluster.id)

          if (isConnected !== wasConnected) {
            console.log(`[ClusterStore] Updating cluster ${cluster.name}: ${wasConnected} -> ${isConnected}`)
            cluster.connected = isConnected
            cluster.clientId = isConnected ? cluster.id : undefined
          }
        }

        console.log('[ClusterStore] After sync:', this.clusters.map(c => ({ id: c.id, name: c.name, connected: c.connected })))
      } catch (e) {
        console.error('[ClusterStore] Failed to sync connection status:', e)
      }
    },

    getClusterByClientId(clientId: string): ClusterItem | undefined {
      return this.clusters.find(c => c.clientId === clientId)
    },

    async restoreClusters(): Promise<string | null> {
      const storedClusters = loadClustersFromStorage()

      for (const cluster of storedClusters) {
        try {
          const client_id = await invoke<string>('connect_cluster', { config: cluster.config })
          const existingIndex = this.clusters.findIndex(c => c.id === cluster.id)
          if (existingIndex >= 0) {
            this.clusters[existingIndex].connected = true
            this.clusters[existingIndex].clientId = client_id
          }
        } catch {
          console.warn(`Failed to restore cluster: ${cluster.name}`)
        }
      }

      // Return first connected cluster's clientId
      const firstConnected = this.clusters.find(c => c.connected)
      return firstConnected?.clientId || null
    },
  },
})
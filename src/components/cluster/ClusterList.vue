<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NList, NListItem, NButton, NSpace, NTag, NEmpty, useMessage } from 'naive-ui'
import { useClusterStore, type ClusterItem } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import { useMessageStore } from '@/stores/message'

const emit = defineEmits<{
  edit: [cluster: ClusterItem]
  add: []
}>()

const clusterStore = useClusterStore()
const tabStore = useTabStore()
const messageStore = useMessageStore()
const message = useMessage()

const connectedClusters = computed(() => clusterStore.clusters)

const handleDisconnect = async (cluster: ClusterItem) => {
  try {
    // Stop real-time consumption if streaming
    if (messageStore.isStreaming && cluster.clientId) {
      await messageStore.stopRealtimeConsume(cluster.clientId, messageStore.streamTopic || '')
    }
    await clusterStore.disconnectCluster(cluster.id)
    message.success('Cluster disconnected successfully')
  } catch (e: unknown) {
    message.error('Failed to disconnect: ' + String(e))
  }
}

const handleSelect = (cluster: ClusterItem) => {
  if (cluster.connected && cluster.clientId) {
    tabStore.addClusterTab({
      clusterId: cluster.id,
      clusterName: cluster.name,
      clientId: cluster.clientId,
      connected: cluster.connected
    })
  }
}

const handleDelete = async (cluster: ClusterItem) => {
  try {
    // Stop real-time consumption if streaming
    if (messageStore.isStreaming && cluster.clientId) {
      await messageStore.stopRealtimeConsume(cluster.clientId, messageStore.streamTopic || '')
    }
    await clusterStore.deleteCluster(cluster.id)
    message.success('Cluster deleted successfully')
  } catch (e: unknown) {
    message.error('Failed to delete: ' + String(e))
  }
}
</script>

<template>
  <n-card title="Clusters">
    <template #header-extra>
      <n-button type="primary" @click="emit('add')">
        + Add Cluster
      </n-button>
    </template>

    <n-empty v-if="connectedClusters.length === 0" description="No clusters configured" />

    <n-list v-else>
      <n-list-item v-for="cluster in connectedClusters" :key="cluster.id">
        <template #prefix>
          <n-tag :type="cluster.connected ? (clusterStore.activeClusterId === cluster.clientId ? 'success' : 'info') : 'default'">
            {{ cluster.connected ? (clusterStore.activeClusterId === cluster.clientId ? 'Active' : 'Connected') : 'Disconnected' }}
          </n-tag>
        </template>
        <n-space justify="space-between" align="center" style="width: 100%">
          <div>
            <strong>{{ cluster.name }}</strong>
            <br />
            <small>{{ cluster.config.bootstrap_servers }}</small>
          </div>
          <n-space>
            <n-button
              v-if="cluster.connected"
              size="small"
              :type="clusterStore.activeClusterId === cluster.clientId ? 'success' : 'default'"
              @click="handleSelect(cluster)"
            >
              Select
            </n-button>
            <n-button size="small" @click="emit('edit', cluster)">Edit</n-button>
            <n-button
              v-if="cluster.connected"
              size="small"
              type="warning"
              @click="handleDisconnect(cluster)"
            >
              Disconnect
            </n-button>
            <n-button
              v-if="!cluster.connected"
              size="small"
              type="primary"
              @click="emit('edit', cluster)"
            >
              Connect
            </n-button>
            <n-button size="small" type="error" @click="handleDelete(cluster)">
              Delete
            </n-button>
          </n-space>
        </n-space>
      </n-list-item>
    </n-list>
  </n-card>
</template>
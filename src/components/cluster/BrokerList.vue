<script setup lang="ts">
import { NCard, NDataTable, NTag } from 'naive-ui'
import { computed, h } from 'vue'
import { useClusterStore } from '@/stores/cluster'

const clusterStore = useClusterStore()

const columns = [
  { title: 'Broker ID', key: 'id' },
  { title: 'Host', key: 'host' },
  { title: 'Port', key: 'port' },
  {
    title: 'Controller',
    key: 'controller',
    render: (row: { id: number }) =>
      row.id === clusterStore.activeClusterInfo?.controller_id
        ? h(NTag, { type: 'success' }, () => 'Controller')
        : null
  },
]

const data = computed(() => clusterStore.activeClusterInfo?.brokers || [])
</script>

<template>
  <n-card title="Brokers" style="margin-top: 16px">
    <n-data-table :columns="columns" :data="data" />
  </n-card>
</template>
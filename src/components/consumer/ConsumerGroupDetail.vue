<script setup lang="ts">
import { onMounted, h } from 'vue'
import { NCard, NDataTable, NTag, NButton, NSpace, NEmpty } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useConsumerStore, type ConsumerGroupMember, type GroupPartitionInfo } from '@/stores/consumer'
import { useClusterStore } from '@/stores/cluster'

const props = defineProps<{
  groupId: string
}>()

const emit = defineEmits<{
  back: []
}>()

const consumerStore = useConsumerStore()
const clusterStore = useClusterStore()

const memberColumns: DataTableColumns<ConsumerGroupMember> = [
  { title: 'Member ID', key: 'member_id', ellipsis: { tooltip: true } },
  { title: 'Client ID', key: 'client_id' },
  { title: 'Client Host', key: 'client_host' },
]

const partitionColumns: DataTableColumns<GroupPartitionInfo> = [
  { title: 'Topic', key: 'topic' },
  { title: 'Partition', key: 'partition', width: 80 },
  { title: 'Current Offset', key: 'current_offset' },
  { title: 'End Offset', key: 'end_offset' },
  {
    title: 'Lag',
    key: 'lag',
    render: (row) => {
      const lag = row.lag
      if (lag < 0) return '-'
      const color = lag === 0 ? 'success' : lag < 1000 ? 'warning' : 'error'
      return h(NTag, { type: color, size: 'small' }, () => lag.toLocaleString())
    },
  },
]

const loadGroupInfo = async () => {
  if (!clusterStore.activeClusterId) return
  await consumerStore.getGroupInfo(clusterStore.activeClusterId, props.groupId)
}

onMounted(() => {
  loadGroupInfo()
})
</script>

<template>
  <n-card :title="`Consumer Group: ${groupId}`">
    <template #header-extra>
      <n-button @click="emit('back')">Back to List</n-button>
    </template>

    <template v-if="consumerStore.selectedGroup">
      <n-space vertical size="large">
        <div>
          <strong>State:</strong>
          <n-tag :type="consumerStore.selectedGroup.state === 'Stable' ? 'success' : 'warning'">
            {{ consumerStore.selectedGroup.state }}
          </n-tag>
        </div>

        <n-card title="Members" size="small">
          <n-empty v-if="consumerStore.selectedGroup.members.length === 0" description="No active members" />
          <n-data-table
            v-else
            :columns="memberColumns"
            :data="consumerStore.selectedGroup.members"
            :pagination="false"
          />
        </n-card>

        <n-card title="Partitions" size="small">
          <n-empty v-if="consumerStore.selectedGroup.partitions.length === 0" description="No partition assignments" />
          <n-data-table
            v-else
            :columns="partitionColumns"
            :data="consumerStore.selectedGroup.partitions"
            :pagination="false"
          />
        </n-card>
      </n-space>
    </template>
  </n-card>
</template>
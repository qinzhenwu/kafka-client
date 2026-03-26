<script setup lang="ts">
import { onMounted } from 'vue'
import { NCard, NList, NListItem, NButton, NSpace, NEmpty } from 'naive-ui'
import { useConsumerStore } from '@/stores/consumer'
import { useClusterStore } from '@/stores/cluster'

const emit = defineEmits<{
  select: [groupId: string]
}>()

const consumerStore = useConsumerStore()
const clusterStore = useClusterStore()

const loadGroups = async () => {
  if (!clusterStore.activeClusterId) return
  await consumerStore.listGroups(clusterStore.activeClusterId)
}

const handleSelect = (groupId: string) => {
  emit('select', groupId)
}

onMounted(() => {
  loadGroups()
})
</script>

<template>
  <n-card title="Consumer Groups">
    <template #header-extra>
      <n-button @click="loadGroups" :loading="consumerStore.loading">
        Refresh
      </n-button>
    </template>

    <n-empty v-if="consumerStore.groups.length === 0" description="No consumer groups found" />

    <n-list v-else>
      <n-list-item v-for="group in consumerStore.groups" :key="group">
        <n-space justify="space-between" align="center" style="width: 100%">
          <div>
            <strong>{{ group }}</strong>
          </div>
          <n-button size="small" @click="handleSelect(group)">
            View Details
          </n-button>
        </n-space>
      </n-list-item>
    </n-list>
  </n-card>
</template>
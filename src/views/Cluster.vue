<script setup lang="ts">
import { ref } from 'vue'
import { NGrid, NGi } from 'naive-ui'
import ClusterList from '@/components/cluster/ClusterList.vue'
import ClusterForm from '@/components/cluster/ClusterForm.vue'
import BrokerList from '@/components/cluster/BrokerList.vue'
import { useClusterStore, type ClusterItem } from '@/stores/cluster'

const clusterStore = useClusterStore()
const showForm = ref(false)
const editingCluster = ref<ClusterItem | null>(null)

const handleAdd = () => {
  editingCluster.value = null
  showForm.value = true
}

const handleEdit = (cluster: ClusterItem) => {
  editingCluster.value = cluster
  showForm.value = true
}
</script>

<template>
  <div>
    <n-grid :cols="1" :x-gap="16">
      <n-gi>
        <ClusterList @add="handleAdd" @edit="handleEdit" />
      </n-gi>

      <n-gi v-if="clusterStore.activeClusterInfo">
        <BrokerList />
      </n-gi>
    </n-grid>

    <ClusterForm
      v-model:show="showForm"
      :edit-cluster="editingCluster"
      @success="showForm = false"
    />
  </div>
</template>
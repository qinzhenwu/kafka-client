<script setup lang="ts">
import { ref, onMounted, computed, h, watch } from 'vue'
import {
  NCard, NDataTable, NButton, NSpace, NInput, NModal, useMessage
} from 'naive-ui'
import type { DataTableColumns, DataTableRowKey } from 'naive-ui'
import { useTopicStore } from '@/stores/topic'
import { useClusterStore } from '@/stores/cluster'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TopicForm from './TopicForm.vue'

const { t } = useI18n()
const router = useRouter()

const topicStore = useTopicStore()
const clusterStore = useClusterStore()
const message = useMessage()

const searchKeyword = ref('')
const showCreateModal = ref(false)
const showEditModal = ref(false)
const showDeleteConfirm = ref(false)
const topicToDelete = ref<string | null>(null)
const editingTopic = ref<{ name: string; partitions: number } | null>(null)

interface TopicRow {
  name: string
}

const filteredTopics = computed<TopicRow[]>(() => {
  if (!searchKeyword.value) {
    return topicStore.topics.map(name => ({ name }))
  }
  const keyword = searchKeyword.value.toLowerCase()
  return topicStore.topics
    .filter(t => t.toLowerCase().includes(keyword))
    .map(name => ({ name }))
})

const columns = computed<DataTableColumns<TopicRow>>(() => [
  {
    title: t('topic.name'),
    key: 'name',
    render: (row) => h('span', { style: { fontWeight: '500' } }, row.name),
  },
  {
    title: t('common.action'),
    key: 'actions',
    width: 280,
    render: (row) =>
      h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', type: 'primary', onClick: () => goToMessages(row.name) }, () => t('topic.viewMessages')),
        h(NButton, { size: 'small', onClick: () => handleEdit(row.name) }, () => t('common.edit')),
        h(NButton, { size: 'small', type: 'error', onClick: () => confirmDelete(row.name) }, () => t('common.delete')),
      ]),
  },
])

const goToMessages = (topicName: string) => {
  router.push({ path: '/message', query: { topic: topicName } })
}

const handleEdit = async (topicName: string) => {
  if (!clusterStore.activeClusterId) return
  try {
    const info = await topicStore.getTopicInfo(clusterStore.activeClusterId, topicName)
    editingTopic.value = {
      name: topicName,
      partitions: info.partitions.length
    }
    showEditModal.value = true
  } catch (e: unknown) {
    message.error('Failed to load topic info: ' + String(e))
  }
}

const loadTopics = async () => {
  if (!clusterStore.activeClusterId) return
  try {
    await topicStore.listTopics(clusterStore.activeClusterId)
  } catch (e: unknown) {
    message.error('Failed to load topics: ' + String(e))
  }
}

const confirmDelete = (topicName: string) => {
  topicToDelete.value = topicName
  showDeleteConfirm.value = true
}

const handleDelete = async () => {
  if (!clusterStore.activeClusterId || !topicToDelete.value) return

  try {
    await topicStore.deleteTopic(clusterStore.activeClusterId, topicToDelete.value)
    message.success('Topic deleted successfully')
  } catch (e: unknown) {
    message.error('Failed to delete topic: ' + String(e))
  } finally {
    showDeleteConfirm.value = false
    topicToDelete.value = null
  }
}

const handleCreateSuccess = () => {
  showCreateModal.value = false
}

const handleEditSuccess = () => {
  showEditModal.value = false
  editingTopic.value = null
}

const getRowKey = (row: TopicRow): DataTableRowKey => row.name

onMounted(() => {
  loadTopics()
})

// Reload topics when active cluster changes
watch(() => clusterStore.activeClusterId, () => {
  if (clusterStore.activeClusterId) {
    loadTopics()
  }
})
</script>

<template>
  <n-card :title="t('topic.title')">
    <template #header-extra>
      <n-space>
        <n-input
          v-model:value="searchKeyword"
          :placeholder="t('topic.searchTopics')"
          clearable
          size="small"
          style="width: 200px"
        />
        <n-button type="primary" size="small" @click="showCreateModal = true">
          + {{ t('topic.createTopic') }}
        </n-button>
      </n-space>
    </template>

    <n-data-table
      :columns="columns"
      :data="filteredTopics"
      :loading="topicStore.loading"
      :row-key="getRowKey"
      :pagination="{ pageSize: 20 }"
    />

    <!-- Create Modal -->
    <topic-form
      v-model:show="showCreateModal"
      @success="handleCreateSuccess"
    />

    <!-- Edit Modal -->
    <topic-form
      v-model:show="showEditModal"
      :edit-topic="editingTopic ?? undefined"
      @success="handleEditSuccess"
    />

    <n-modal
      v-model:show="showDeleteConfirm"
      preset="confirm"
      :title="t('topic.confirmDelete')"
      :content="t('topic.deleteConfirmText', { name: topicToDelete })"
      type="error"
      @positive-click="handleDelete"
      @negative-click="showDeleteConfirm = false"
    />
  </n-card>
</template>
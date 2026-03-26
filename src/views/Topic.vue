<script setup lang="ts">
import { ref, watch } from 'vue'
import { NCard, NSpace, NDescriptions, NDescriptionsItem, NTag, NButton, NDivider, useMessage } from 'naive-ui'
import { useTopicStore } from '@/stores/topic'
import { useClusterStore } from '@/stores/cluster'
import TopicList from '@/components/topic/TopicList.vue'

const topicStore = useTopicStore()
const clusterStore = useClusterStore()
const message = useMessage()

const selectedTopicName = ref<string | null>(null)

const handleTopicSelect = async (topicName: string) => {
  selectedTopicName.value = topicName
  if (clusterStore.activeClusterId) {
    try {
      await topicStore.getTopicInfo(clusterStore.activeClusterId, topicName)
    } catch (e: unknown) {
      message.error('Failed to load topic info: ' + String(e))
    }
  }
}

const handleBack = () => {
  selectedTopicName.value = null
  topicStore.clearSelectedTopic()
}

// Watch for cluster change to clear selected topic
watch(() => clusterStore.activeClusterId, () => {
  selectedTopicName.value = null
  topicStore.clearSelectedTopic()
})
</script>

<template>
  <n-space vertical size="large">
    <topic-list
      v-if="!selectedTopicName"
      @select="handleTopicSelect"
    />

    <template v-else>
      <n-button @click="handleBack">&larr; Back to Topics</n-button>

      <n-card :title="selectedTopicName">
        <template v-if="topicStore.selectedTopic">
          <n-descriptions label-placement="left" :column="2" bordered>
            <n-descriptions-item label="Topic Name">
              {{ topicStore.selectedTopic.name }}
            </n-descriptions-item>
            <n-descriptions-item label="Partitions">
              {{ topicStore.selectedTopic.partitions.length }}
            </n-descriptions-item>
          </n-descriptions>

          <n-divider>Partition Details</n-divider>

          <n-space vertical>
            <n-card
              v-for="partition in topicStore.selectedTopic.partitions"
              :key="partition.id"
              size="small"
              :title="`Partition ${partition.id}`"
            >
              <n-descriptions label-placement="left" :column="2">
                <n-descriptions-item label="Leader">
                  <n-tag type="success">{{ partition.leader }}</n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="Replicas">
                  <n-space>
                    <n-tag v-for="r in partition.replicas" :key="r" type="info">{{ r }}</n-tag>
                  </n-space>
                </n-descriptions-item>
                <n-descriptions-item label="ISR">
                  <n-space>
                    <n-tag v-for="i in partition.isr" :key="i" type="success">{{ i }}</n-tag>
                  </n-space>
                </n-descriptions-item>
              </n-descriptions>
            </n-card>
          </n-space>
        </template>
      </n-card>
    </template>
  </n-space>
</template>
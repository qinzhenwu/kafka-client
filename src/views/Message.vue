<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { NCard, NSpace, NSelect, NGrid, NGi, NEmpty } from 'naive-ui'
import { useRoute } from 'vue-router'
import ProduceForm from '@/components/message/ProduceForm.vue'
import MessageList from '@/components/message/MessageList.vue'
import { useClusterStore } from '@/stores/cluster'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const route = useRoute()
const clusterStore = useClusterStore()

const selectedTopic = ref<string | null>(null)
const topics = ref<string[]>([])
const loading = ref(false)

const activeTab = ref<'consume' | 'produce'>('consume')

const loadTopics = async () => {
  if (!clusterStore.activeClusterId) return

  loading.value = true
  try {
    topics.value = await invoke<string[]>('list_topics', {
      clientId: clusterStore.activeClusterId,
    })
  } catch (e) {
    console.error('Failed to load topics:', e)
  } finally {
    loading.value = false
  }
}

const topicOptions = computed(() =>
  topics.value.map((t) => ({ label: t, value: t }))
)

// Load topics when cluster is active
onMounted(() => {
  if (clusterStore.activeClusterId) {
    loadTopics().then(() => {
      // Check for topic query parameter after topics are loaded
      if (route.query.topic) {
        const topicFromQuery = route.query.topic as string
        if (topics.value.includes(topicFromQuery)) {
          selectedTopic.value = topicFromQuery
        }
      }
    })
  }
})

// Watch for URL query changes
watch(() => route.query.topic, (newTopic) => {
  if (newTopic && typeof newTopic === 'string') {
    if (topics.value.includes(newTopic)) {
      selectedTopic.value = newTopic
    }
  }
})
</script>

<template>
  <div>
    <n-grid :cols="1" :x-gap="16">
      <n-gi>
        <n-card>
          <n-space align="center">
            <n-select
              v-model:value="selectedTopic"
              :options="topicOptions"
              :placeholder="t('message.selectTopic')"
              filterable
              size="small"
              :loading="loading"
              style="width: 300px"
              @update:show="(show: boolean) => show && loadTopics()"
            />
          </n-space>
        </n-card>
      </n-gi>

      <n-gi v-if="selectedTopic">
        <n-card>
          <template #header>
            <n-space>
              <span
                :style="{ cursor: 'pointer', fontWeight: activeTab === 'consume' ? 'bold' : 'normal' }"
                @click="activeTab = 'consume'"
              >
                {{ t('message.consume') }}
              </span>
              <span>|</span>
              <span
                :style="{ cursor: 'pointer', fontWeight: activeTab === 'produce' ? 'bold' : 'normal' }"
                @click="activeTab = 'produce'"
              >
                {{ t('message.produce') }}
              </span>
            </n-space>
          </template>

          <MessageList v-if="activeTab === 'consume'" :topic-name="selectedTopic" />
          <ProduceForm v-else :topic-name="selectedTopic" @success="activeTab = 'consume'" />
        </n-card>
      </n-gi>

      <n-gi v-else>
        <n-empty :description="t('message.selectTopicHint')" />
      </n-gi>
    </n-grid>
  </div>
</template>
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface PartitionInfo {
  id: number
  leader: number
  replicas: number[]
  isr: number[]
  high_watermark: number
  low_watermark: number
}

export interface TopicInfo {
  name: string
  partitions: PartitionInfo[]
  configs: { key: string; value: string }[]
}

export interface CreateTopicRequest {
  name: string
  num_partitions: number
  replication_factor: number
  configs: Record<string, string>
}

interface TopicState {
  topics: string[]
  selectedTopic: TopicInfo | null
  loading: boolean
  error: string | null
}

export const useTopicStore = defineStore('topic', {
  state: (): TopicState => ({
    topics: [],
    selectedTopic: null,
    loading: false,
    error: null,
  }),

  actions: {
    async listTopics(clientId: string) {
      this.loading = true
      this.error = null
      try {
        const topics = await invoke<string[]>('list_topics', { clientId })
        this.topics = topics
        return topics
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async getTopicInfo(clientId: string, topicName: string) {
      this.loading = true
      this.error = null
      try {
        const info = await invoke<TopicInfo>('get_topic_info', { clientId, topicName })
        this.selectedTopic = info
        return info
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async createTopic(clientId: string, request: CreateTopicRequest) {
      this.loading = true
      this.error = null
      try {
        await invoke('create_topic', { clientId, request })
        // Refresh topic list
        await this.listTopics(clientId)
        return true
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async updateTopicPartitions(clientId: string, topicName: string, numPartitions: number) {
      this.loading = true
      this.error = null
      try {
        await invoke('update_topic_partitions', { clientId, topicName, numPartitions })
        // Refresh topic info
        await this.getTopicInfo(clientId, topicName)
        return true
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async deleteTopic(clientId: string, topicName: string) {
      this.loading = true
      this.error = null
      try {
        await invoke('delete_topic', { clientId, topicName })
        // Refresh topic list
        await this.listTopics(clientId)
        return true
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    clearSelectedTopic() {
      this.selectedTopic = null
    },
  },
})
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface KafkaMessage {
  partition: number
  offset: number
  key: string | null
  value: string | null
  headers: Record<string, string>
  timestamp: string
}

export interface ProduceRequest {
  topic: string
  partition?: number
  key?: string
  value: string
  headers: Record<string, string>
}

interface MessageState {
  messages: KafkaMessage[]
  loading: boolean
  error: string | null
  hasMore: boolean
  isStreaming: boolean
  streamTopic: string | null
  maxMessages: number
  unlisten: UnlistenFn | null
  // Produce dialog state
  showProduceDialog: boolean
  produceDialogTopic: string | null
}

// Sort messages by timestamp descending (newest first)
const sortMessagesByTimestampDesc = (messages: KafkaMessage[]): KafkaMessage[] => {
  return [...messages].sort((a, b) => {
    const timeA = new Date(a.timestamp).getTime()
    const timeB = new Date(b.timestamp).getTime()
    return timeB - timeA
  })
}

export const useMessageStore = defineStore('message', {
  state: (): MessageState => ({
    messages: [],
    loading: false,
    error: null,
    hasMore: false,
    isStreaming: false,
    streamTopic: null,
    maxMessages: 1000,
    unlisten: null,
    showProduceDialog: false,
    produceDialogTopic: null,
  }),

  actions: {
    async produceMessage(clientId: string, request: ProduceRequest) {
      this.loading = true
      this.error = null
      try {
        await invoke('produce_message', { clientId, request })
        return true
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async consumeMessages(clientId: string, topic: string, limit: number = 100) {
      this.loading = true
      this.error = null
      try {
        const response = await invoke<{ messages: KafkaMessage[]; has_more: boolean }>(
          'consume_messages',
          {
            clientId,
            request: {
              topic,
              limit,
            },
          }
        )
        this.messages = sortMessagesByTimestampDesc(response.messages)
        this.hasMore = response.has_more
        return response.messages
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async consumeFromOffset(
      clientId: string,
      topic: string,
      partition: number,
      offset: number,
      limit: number = 100
    ) {
      this.loading = true
      this.error = null
      try {
        const response = await invoke<{ messages: KafkaMessage[]; has_more: boolean }>(
          'consume_from_offset',
          {
            clientId,
            topic,
            partition,
            offset,
            limit,
          }
        )
        this.messages = sortMessagesByTimestampDesc(response.messages)
        this.hasMore = response.has_more
        return response.messages
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async consumeFromPosition(
      clientId: string,
      topic: string,
      position: string,
      limit: number = 100
    ) {
      this.loading = true
      this.error = null
      try {
        const response = await invoke<{ messages: KafkaMessage[]; has_more: boolean }>(
          'consume_from_position',
          {
            clientId,
            topic,
            position,
            limit,
          }
        )
        this.messages = sortMessagesByTimestampDesc(response.messages)
        this.hasMore = response.has_more
        return response.messages
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async startRealtimeConsume(clientId: string, topic: string, position: 'earliest' | 'latest') {
      // Stop existing streaming if any
      if (this.isStreaming && this.streamTopic) {
        await this.stopRealtimeConsume(clientId, this.streamTopic)
      }

      // Clear messages
      this.messages = []

      // Set up event listener
      this.unlisten = await listen<KafkaMessage>('kafka-message', (event) => {
        this.addMessage(event.payload)
      })

      // Start streaming
      try {
        await invoke('start_realtime_consume', { clientId, topic, position })
        this.isStreaming = true
        this.streamTopic = topic
      } catch (e: unknown) {
        this.error = String(e)
        if (this.unlisten) {
          this.unlisten()
          this.unlisten = null
        }
        throw e
      }
    },

    async stopRealtimeConsume(clientId: string, topic: string) {
      try {
        await invoke('stop_realtime_consume', { clientId, topic })
      } finally {
        this.isStreaming = false
        this.streamTopic = null
        if (this.unlisten) {
          this.unlisten()
          this.unlisten = null
        }
      }
    },

    addMessage(message: KafkaMessage) {
      // Insert message in the correct position to maintain descending order
      const newTime = new Date(message.timestamp).getTime()
      let insertIndex = 0

      // Find the correct position to insert
      for (let i = 0; i < this.messages.length; i++) {
        const existingTime = new Date(this.messages[i].timestamp).getTime()
        if (newTime > existingTime) {
          insertIndex = i
          break
        }
        insertIndex = i + 1
      }

      this.messages.splice(insertIndex, 0, message)

      // Keep only the latest maxMessages
      if (this.messages.length > this.maxMessages) {
        this.messages = this.messages.slice(0, this.maxMessages)
      }
    },

    clearMessages() {
      this.messages = []
      this.hasMore = false
    },

    openProduceDialog(topic: string) {
      this.showProduceDialog = true
      this.produceDialogTopic = topic
    },

    closeProduceDialog() {
      this.showProduceDialog = false
      this.produceDialogTopic = null
    },
  },
})
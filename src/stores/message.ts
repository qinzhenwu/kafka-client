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
  // Cached timestamp number for efficient sorting (optional, added during processing)
  timestampNum?: number
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

      // Set up event listener for batch messages
      this.unlisten = await listen<KafkaMessage[]>('kafka-message-batch', (event) => {
        this.addMessageBatch(event.payload)
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
      // Single message uses batch processing logic
      this.addMessageBatch([message])
    },

    /**
     * Batch add messages using merge algorithm for O(n) performance.
     * Messages are kept in descending timestamp order with maxMessages limit.
     */
    addMessageBatch(messages: KafkaMessage[]) {
      if (messages.length === 0) return

      // Pre-process: calculate and cache timestamps, sort descending
      const processed = messages
        .map(msg => ({
          ...msg,
          timestampNum: new Date(msg.timestamp).getTime()
        }))
        .sort((a, b) => b.timestampNum! - a.timestampNum!)

      // If current array is empty, directly assign
      if (this.messages.length === 0) {
        this.messages = processed.slice(0, this.maxMessages)
        return
      }

      // Merge two sorted arrays (both in descending order)
      // Prepare existing messages with cached timestamps
      const existing = this.messages.map(msg => ({
        ...msg,
        timestampNum: msg.timestampNum ?? new Date(msg.timestamp).getTime()
      }))

      const merged: KafkaMessage[] = []
      let i = 0 // new messages index
      let j = 0 // existing messages index

      // Merge while respecting maxMessages limit
      while (i < processed.length && j < existing.length && merged.length < this.maxMessages) {
        if (processed[i].timestampNum! >= existing[j].timestampNum!) {
          merged.push(processed[i])
          i++
        } else {
          merged.push(existing[j])
          j++
        }
      }

      // Add remaining elements from processed (if space available)
      while (i < processed.length && merged.length < this.maxMessages) {
        merged.push(processed[i])
        i++
      }

      // Add remaining elements from existing (if space available)
      while (j < existing.length && merged.length < this.maxMessages) {
        merged.push(existing[j])
        j++
      }

      // Single Vue reactive update
      this.messages = merged
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
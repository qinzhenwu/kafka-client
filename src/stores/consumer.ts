import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface ConsumerGroupMember {
  member_id: string
  client_id: string
  client_host: string
  assignment: number[]
}

export interface GroupPartitionInfo {
  topic: string
  partition: number
  current_offset: number
  end_offset: number
  lag: number
  member_id: string | null
}

export interface ConsumerGroupInfo {
  group_id: string
  state: string
  members: ConsumerGroupMember[]
  partitions: GroupPartitionInfo[]
}

export interface LagInfo {
  topic: string
  partition: number
  current_offset: number
  end_offset: number
  lag: number
}

// Match Rust serde output with rename_all = "snake_case"
export type ResetOffsetType =
  | { type: 'earliest' }
  | { type: 'latest' }
  | { type: 'timestamp'; value: number }
  | { type: 'specific'; value: number }

interface ConsumerState {
  groups: string[]
  selectedGroup: ConsumerGroupInfo | null
  lagInfo: LagInfo[]
  loading: boolean
  error: string | null
}

export const useConsumerStore = defineStore('consumer', {
  state: (): ConsumerState => ({
    groups: [],
    selectedGroup: null,
    lagInfo: [],
    loading: false,
    error: null,
  }),

  actions: {
    async listGroups(clientId: string) {
      this.loading = true
      this.error = null
      try {
        this.groups = await invoke<string[]>('list_consumer_groups', { clientId })
        return this.groups
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async getGroupInfo(clientId: string, groupId: string) {
      this.loading = true
      this.error = null
      try {
        this.selectedGroup = await invoke<ConsumerGroupInfo>('get_consumer_group_info', {
          clientId,
          groupId,
        })
        return this.selectedGroup
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async getGroupLag(clientId: string, groupId: string) {
      this.loading = true
      this.error = null
      try {
        this.lagInfo = await invoke<LagInfo[]>('get_group_lag', {
          clientId,
          groupId,
        })
        return this.lagInfo
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    async resetOffsets(
      clientId: string,
      groupId: string,
      topic: string,
      partitions: number[],
      resetType: ResetOffsetType
    ) {
      this.loading = true
      this.error = null
      try {
        await invoke('reset_consumer_group_offsets', {
          clientId,
          groupId,
          topic,
          partitions,
          resetType,
        })
      } catch (e: unknown) {
        this.error = String(e)
        throw e
      } finally {
        this.loading = false
      }
    },

    clearSelection() {
      this.selectedGroup = null
      this.lagInfo = []
    },
  },
})
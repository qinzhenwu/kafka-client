<!-- src/components/consumer/ResetOffsetsModal.vue -->
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NSelect, NCheckbox, NInputNumber, NDatePicker, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useClusterStore } from '@/stores/cluster'
import { useConsumerStore, type ResetOffsetType } from '@/stores/consumer'
import { X } from 'lucide-vue-next'

const props = defineProps<{
  show: boolean
  groupId: string
  topics: string[]  // List of subscribed topics from parent
  partitionCounts: Record<string, number>  // Map of topic -> partition count
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  success: []
}>()

const { t } = useI18n()
const message = useMessage()
const clusterStore = useClusterStore()
const consumerStore = useConsumerStore()

const resetType = ref<'earliest' | 'latest' | 'specific' | 'timestamp'>('latest')
const selectedTopic = ref<string | null>(null)
const allPartitions = ref(true)
const specificPartitions = ref<number[]>([])
const specificOffset = ref(0)
const timestamp = ref(Date.now())

const resetTypeOptions = [
  { label: t('consumer.earliest'), value: 'earliest' },
  { label: t('consumer.latest'), value: 'latest' },
  { label: t('consumer.specificOffset'), value: 'specific' },
  { label: t('consumer.timestamp'), value: 'timestamp' },
]

const topicOptions = computed(() =>
  props.topics.map(topic => ({ label: topic, value: topic }))
)

const partitionOptions = computed(() => {
  if (!selectedTopic.value) return []
  const count = props.partitionCounts[selectedTopic.value] || 0
  return Array.from({ length: count }, (_, i) => ({ label: String(i), value: i }))
})

const loading = ref(false)

// Reset selections when topic changes
watch(selectedTopic, () => {
  allPartitions.value = true
  specificPartitions.value = []
})

const getResetTypePayload = (): ResetOffsetType => {
  switch (resetType.value) {
    case 'earliest':
      return { type: 'earliest' }
    case 'latest':
      return { type: 'latest' }
    case 'specific':
      return { type: 'specific', value: specificOffset.value }
    case 'timestamp':
      return { type: 'timestamp', value: timestamp.value }
  }
}

const handleReset = async () => {
  if (!clusterStore.activeClusterId || !selectedTopic.value) {
    message.warning(t('consumer.selectTopic'))
    return
  }

  const partitionsToReset = allPartitions.value ? [] : specificPartitions.value

  loading.value = true
  try {
    await consumerStore.resetOffsets(
      clusterStore.activeClusterId,
      props.groupId,
      selectedTopic.value,
      partitionsToReset,
      getResetTypePayload()
    )
    message.success(t('consumer.resetSuccess'))
    emit('success')
    emit('update:show', false)
  } catch (e: unknown) {
    message.error(t('consumer.resetFailed') + ': ' + String(e))
  } finally {
    loading.value = false
  }
}

const handleClose = () => {
  emit('update:show', false)
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleClose">
      <div class="modal-container" @click.stop>
        <div class="modal-header">
          <span class="modal-title">{{ t('consumer.resetOffsets') }}</span>
          <X :size="18" class="modal-close" @click="handleClose" />
        </div>
        <div class="modal-content">
          <div class="reset-form">
            <!-- Topic Selection -->
            <div class="form-row">
              <label class="form-label">{{ t('consumer.selectTopic') }}</label>
              <n-select
                v-model:value="selectedTopic"
                :options="topicOptions"
                :placeholder="t('consumer.selectTopic')"
                size="small"
              />
            </div>

            <!-- Reset Type -->
            <div class="form-row">
              <label class="form-label">{{ t('consumer.resetType') }}</label>
              <n-select
                v-model:value="resetType"
                :options="resetTypeOptions"
                size="small"
              />
            </div>

            <!-- Specific Offset Input -->
            <div v-if="resetType === 'specific'" class="form-row">
              <label class="form-label">{{ t('consumer.specificOffset') }}</label>
              <n-input-number v-model:value="specificOffset" :min="0" size="small" style="width: 100%" />
            </div>

            <!-- Timestamp Picker -->
            <div v-if="resetType === 'timestamp'" class="form-row">
              <label class="form-label">{{ t('consumer.timestamp') }}</label>
              <n-date-picker v-model:value="timestamp" type="datetime" size="small" style="width: 100%" />
            </div>

            <!-- Partition Selection -->
            <div v-if="selectedTopic" class="form-row">
              <label class="form-label">{{ t('consumer.selectPartitions') }}</label>
              <div class="partition-options">
                <n-checkbox v-model:checked="allPartitions">
                  {{ t('consumer.allPartitions') }}
                </n-checkbox>
                <n-select
                  v-if="!allPartitions"
                  v-model:value="specificPartitions"
                  :options="partitionOptions"
                  multiple
                  size="small"
                  :placeholder="t('consumer.selectPartitions')"
                />
              </div>
            </div>

            <div class="warning-box">
              {{ t('consumer.resetWarning') }}
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-cancel" @click="handleClose">{{ t('common.cancel') }}</button>
          <button class="btn btn-confirm" :disabled="loading || !selectedTopic" @click="handleReset">
            {{ t('consumer.confirmReset') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  width: 480px;
  max-height: 80vh;
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.modal-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.modal-close {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.modal-close:hover {
  color: var(--text-primary);
}

.modal-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.reset-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.form-label {
  width: 100px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
  padding-top: 6px;
}

.partition-options {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.warning-box {
  background: rgba(250, 173, 20, 0.1);
  border: 1px solid var(--warning);
  border-radius: 6px;
  padding: 12px;
  font-size: 12px;
  color: var(--warning);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-cancel {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.btn-cancel:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-confirm {
  background: var(--accent);
  color: white;
}

.btn-confirm:hover {
  background: var(--accent-hover);
}

.btn-confirm:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
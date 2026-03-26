<script setup lang="ts">
import { ref, watch } from 'vue'
import { NInput, NInputNumber, useMessage } from 'naive-ui'
import { useTopicStore } from '@/stores/topic'
import { useClusterStore } from '@/stores/cluster'
import { useI18n } from 'vue-i18n'
import { Plus, Pencil, AlertTriangle, X } from 'lucide-vue-next'

const { t } = useI18n()
const props = defineProps<{
  show: boolean
  editTopic?: { name: string; partitions: number } | undefined
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  success: []
}>()

const topicStore = useTopicStore()
const clusterStore = useClusterStore()
const message = useMessage()

const isEditMode = ref(false)
const currentPartitions = ref(0)

const formValue = ref({
  name: '',
  num_partitions: 1,
  replication_factor: 1,
})

// Watch for edit mode changes
watch(() => props.editTopic, (topic) => {
  if (topic) {
    isEditMode.value = true
    currentPartitions.value = topic.partitions
    formValue.value = {
      name: topic.name,
      num_partitions: topic.partitions,
      replication_factor: 1,
    }
  } else {
    isEditMode.value = false
    currentPartitions.value = 0
  }
}, { immediate: true })

// Reset form when modal closes
watch(() => props.show, (show) => {
  if (!show) {
    resetForm()
  }
})

const handleCreate = async () => {
  if (!clusterStore.activeClusterId) return

  try {
    if (isEditMode.value) {
      // Edit mode: update partitions
      if (formValue.value.num_partitions > currentPartitions.value) {
        await topicStore.updateTopicPartitions(
          clusterStore.activeClusterId,
          formValue.value.name,
          formValue.value.num_partitions
        )
        message.success(t('topic.updateSuccess', 'Topic partitions updated successfully'))
      } else {
        message.info(t('topic.noChanges', 'No changes to update'))
      }
    } else {
      // Create mode
      await topicStore.createTopic(clusterStore.activeClusterId, {
        name: formValue.value.name,
        num_partitions: formValue.value.num_partitions,
        replication_factor: formValue.value.replication_factor,
        configs: {},
      })
      message.success(t('topic.createSuccess', 'Topic created successfully'))
    }
    emit('success')
    emit('update:show', false)
    resetForm()
  } catch (e: unknown) {
    const errorMsg = String(e)
    if (isEditMode.value) {
      message.error(t('topic.updateFailed', 'Failed to update topic partitions') + ': ' + errorMsg)
    } else {
      message.error(t('topic.createFailed', 'Failed to create topic') + ': ' + errorMsg)
    }
  }
}

const resetForm = () => {
  formValue.value = {
    name: '',
    num_partitions: 1,
    replication_factor: 1,
  }
  isEditMode.value = false
  currentPartitions.value = 0
}

// Validation for edit mode
const canSubmit = () => {
  if (isEditMode.value) {
    return formValue.value.num_partitions >= currentPartitions.value
  }
  return formValue.value.name.length > 0
}

const handleClose = () => {
  emit('update:show', false)
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click="handleClose">
    <div class="modal-container" @click.stop>
      <!-- Header -->
      <div class="modal-header">
        <span class="header-title">
          <component :is="isEditMode ? Pencil : Plus" :size="18" :stroke-width="1.5" class="header-icon" />
          {{ isEditMode ? t('topic.editTopic', 'Edit Topic') : t('topic.createTopic') }}
        </span>
        <X :size="16" class="header-close" @click="handleClose" />
      </div>

      <!-- Content -->
      <div class="modal-content">
        <!-- Topic Name -->
        <div class="form-row">
          <label class="form-label">{{ t('topic.topicName') }}<span class="required">*</span></label>
          <n-input
            v-model:value="formValue.name"
            placeholder="my-topic"
            :disabled="isEditMode"
            size="small"
          />
        </div>

        <!-- Partitions -->
        <div class="form-row">
          <label class="form-label">{{ t('topic.partitions') }}<span class="required">*</span></label>
          <div class="partition-input">
            <n-input-number
              v-model:value="formValue.num_partitions"
              :min="isEditMode ? currentPartitions : 1"
              :max="100"
              size="small"
              style="width: 100%"
            />
            <div v-if="isEditMode" class="partition-hint">
              {{ t('topic.currentPartitions', 'Current') }}: {{ currentPartitions }}
              <span class="hint-note">({{ t('topic.canOnlyIncrease', 'can only increase') }})</span>
            </div>
          </div>
        </div>

        <!-- Replication Factor (only for create) -->
        <div v-if="!isEditMode" class="form-row">
          <label class="form-label">{{ t('topic.replicationFactor') }}<span class="required">*</span></label>
          <n-input-number
            v-model:value="formValue.replication_factor"
            :min="1"
            :max="10"
            size="small"
            style="width: 100%"
          />
        </div>

        <!-- Warning for invalid partition decrease -->
        <div v-if="isEditMode && formValue.num_partitions < currentPartitions" class="warning-box">
          <AlertTriangle :size="14" class="warning-icon" />
          <span>{{ t('topic.cannotDecreasePartitions', 'Cannot decrease partitions') }}</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <span class="footer-hint">{{ isEditMode ? t('topic.editTopicHint', 'Modify topic partitions') : t('topic.createTopicHint', 'Create a new topic') }}</span>
        <div class="footer-actions">
          <button class="action-btn" @click="handleClose">{{ t('common.cancel') }}</button>
          <button
            class="action-btn primary"
            :disabled="!canSubmit()"
            @click="handleCreate"
          >
            {{ isEditMode ? t('common.save') : t('common.create') }}
          </button>
        </div>
      </div>
    </div>
  </div>
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
  background: var(--bg-secondary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-icon {
  color: var(--text-primary);
}

.header-close {
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.header-close:hover {
  color: var(--text-primary);
}

.modal-content {
  padding: 20px;
}

.form-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

.form-row:last-child {
  margin-bottom: 0;
}

.form-label {
  width: 100px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
  padding-top: 6px;
}

.required {
  color: var(--error);
  margin-left: 2px;
}

.partition-input {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.partition-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.hint-note {
  color: var(--warning);
}

.warning-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(250, 173, 20, 0.1);
  border: 1px solid var(--warning);
  border-radius: 6px;
  font-size: 13px;
  color: var(--warning);
  margin-top: 16px;
}

.warning-icon {
  flex-shrink: 0;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
}

.footer-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: none;
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.primary {
  background: var(--accent);
  color: white;
}

.action-btn.primary:hover {
  background: var(--accent-hover);
}

.action-btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
<script setup lang="ts">
import { ref } from 'vue'
import {
  NInput, NInputNumber, NSelect, useMessage
} from 'naive-ui'
import { useMessageStore, type ProduceRequest } from '@/stores/message'
import { useClusterStore } from '@/stores/cluster'
import { useI18n } from 'vue-i18n'
import IconButton from '@/components/common/IconButton.vue'
import { Send, X } from 'lucide-vue-next'

const props = defineProps<{
  topicName: string
}>()

const emit = defineEmits<{
  success: []
}>()

const { t } = useI18n()
const messageStore = useMessageStore()
const clusterStore = useClusterStore()
const message = useMessage()

const formValue = ref<ProduceRequest>({
  topic: props.topicName,
  partition: undefined,
  key: '',
  value: '',
  headers: {},
})

const headerKey = ref('')
const headerValue = ref('')
const messageFormat = ref<'json' | 'text'>('text')

const addHeader = () => {
  if (headerKey.value && headerValue.value) {
    formValue.value.headers[headerKey.value] = headerValue.value
    headerKey.value = ''
    headerValue.value = ''
  }
}

const removeHeader = (key: string) => {
  delete formValue.value.headers[key]
}

const formatJson = () => {
  try {
    const parsed = JSON.parse(formValue.value.value)
    formValue.value.value = JSON.stringify(parsed, null, 2)
  } catch {
    // Not valid JSON, ignore
  }
}

const handleProduce = async () => {
  if (!clusterStore.activeClusterId) return

  try {
    await messageStore.produceMessage(clusterStore.activeClusterId, {
      ...formValue.value,
      key: formValue.value.key || undefined,
      partition: formValue.value.partition,
    })
    message.success(t('message.produceSuccess', 'Message produced successfully'))
    emit('success')
    // Reset form
    formValue.value = {
      topic: props.topicName,
      partition: undefined,
      key: '',
      value: '',
      headers: {},
    }
  } catch (e: unknown) {
    message.error(t('message.produceError', 'Failed to produce message') + ': ' + String(e))
  }
}
</script>

<template>
  <div class="produce-form">
    <!-- Header -->
    <div class="form-header">
      <div class="header-title">
        <Send :size="20" :stroke-width="1.5" class="title-icon" />
        <span class="title-text">{{ t('message.produceMessage', 'Produce Message') }}</span>
      </div>
      <div class="header-actions">
        <IconButton
          icon="send"
          :tooltip="t('message.send', 'Send')"
          size="small"
          @click="handleProduce"
        />
      </div>
    </div>

    <!-- Form Content -->
    <div class="form-content">
      <!-- Topic (readonly) -->
      <div class="form-row">
        <label class="form-label">{{ t('message.topic', 'Topic') }}</label>
        <div class="form-value readonly">{{ formValue.topic }}</div>
      </div>

      <!-- Partition -->
      <div class="form-row">
        <label class="form-label">{{ t('message.partition', 'Partition') }}</label>
        <n-input-number
          v-model:value="formValue.partition"
          :min="0"
          :placeholder="t('message.auto', 'Auto')"
          clearable
          size="small"
          style="width: 120px"
        />
      </div>

      <!-- Key -->
      <div class="form-row">
        <label class="form-label">{{ t('message.key', 'Key') }}</label>
        <n-input
          v-model:value="formValue.key"
          :placeholder="t('message.keyPlaceholder', 'Message key (optional)')"
          size="small"
        />
      </div>

      <!-- Format -->
      <div class="form-row">
        <label class="form-label">{{ t('message.format', 'Format') }}</label>
        <div class="format-group">
          <n-select
            v-model:value="messageFormat"
            :options="[
              { label: 'Plain Text', value: 'text' },
              { label: 'JSON', value: 'json' },
            ]"
            size="small"
            style="width: 120px"
          />
          <IconButton
            v-if="messageFormat === 'json'"
            icon="clipboard"
            :tooltip="t('message.formatJson', 'Format JSON')"
            size="small"
            @click="formatJson"
          />
        </div>
      </div>

      <!-- Value -->
      <div class="form-row form-row-value">
        <label class="form-label">{{ t('message.value', 'Value') }}</label>
        <n-input
          v-model:value="formValue.value"
          type="textarea"
          :rows="8"
          :placeholder="t('message.valuePlaceholder', 'Message value')"
        />
      </div>

      <!-- Headers -->
      <div class="headers-section">
        <div class="section-title">{{ t('message.headers', 'Headers') }} ({{ t('common.optional', 'Optional') }})</div>

        <!-- Existing headers -->
        <div v-if="Object.keys(formValue.headers).length > 0" class="headers-list">
          <div v-for="(value, key) in formValue.headers" :key="key" class="header-item">
            <span class="header-key">{{ key }}</span>
            <span class="header-value">{{ value }}</span>
            <X :size="14" class="remove-header-icon" @click="removeHeader(key as string)" />
          </div>
        </div>

        <!-- Add header -->
        <div class="add-header-row">
          <n-input
            v-model:value="headerKey"
            :placeholder="t('message.headerKey', 'Key')"
            size="small"
            style="width: 150px"
          />
          <n-input
            v-model:value="headerValue"
            :placeholder="t('message.headerValue', 'Value')"
            size="small"
            style="flex: 1"
          />
          <IconButton
            icon="add"
            :tooltip="t('message.addHeader', 'Add')"
            size="small"
            @click="addHeader"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.produce-form {
  display: flex;
  flex-direction: column;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.title-icon {
  color: var(--text-primary);
}

.title-text {
  font-size: 18px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.form-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.form-row-value {
  align-items: flex-start;
}

.form-label {
  width: 80px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.form-value {
  font-size: 13px;
  color: var(--text-primary);
}

.form-value.readonly {
  color: var(--text-muted);
  padding: 6px 0;
}

.format-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.headers-section {
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.section-title {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.headers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.header-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.header-key {
  font-size: 13px;
  color: var(--accent);
  font-weight: 500;
  min-width: 80px;
}

.header-value {
  font-size: 13px;
  color: var(--text-secondary);
  flex: 1;
}

.remove-header-icon {
  color: var(--text-muted);
  cursor: pointer;
  transition: color 0.15s ease;
}

.remove-header-icon:hover {
  color: var(--error);
}

.add-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

</style>
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NInput, NSelect, useMessage } from 'naive-ui'
import { useClusterStore, type ClusterConfig, type ClusterItem, type ClusterInfo } from '@/stores/cluster'
import { useTabStore } from '@/stores/tabs'
import { useI18n } from 'vue-i18n'
import { Plus, Pencil, X, Zap, Check } from 'lucide-vue-next'

const { t } = useI18n()
const props = defineProps<{
  show: boolean
  editCluster?: ClusterItem | null
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  success: []
}>()

const clusterStore = useClusterStore()
const tabStore = useTabStore()
const message = useMessage()

const defaultFormValue = (): Partial<ClusterConfig> => ({
  name: '',
  bootstrap_servers: '',
  security_protocol: 'PLAINTEXT',
  sasl_mechanism: undefined,
  sasl_username: '',
  sasl_password: '',
})

const formValue = ref<Partial<ClusterConfig>>(defaultFormValue())
const testingConnection = ref(false)
const connectionTested = ref(false)
const testedClusterInfo = ref<ClusterInfo | null>(null)

// Watch for editCluster changes to populate form
watch(
  () => props.editCluster,
  (cluster) => {
    if (cluster) {
      formValue.value = {
        id: cluster.config.id,
        name: cluster.config.name,
        bootstrap_servers: cluster.config.bootstrap_servers,
        security_protocol: cluster.config.security_protocol,
        sasl_mechanism: cluster.config.sasl_mechanism,
        sasl_username: cluster.config.sasl_username,
        sasl_password: cluster.config.sasl_password,
        ssl_ca_location: cluster.config.ssl_ca_location,
        ssl_certificate_location: cluster.config.ssl_certificate_location,
        ssl_key_location: cluster.config.ssl_key_location,
        ssl_key_password: cluster.config.ssl_key_password,
      }
    } else {
      formValue.value = defaultFormValue()
    }
    // Reset test state when form changes
    connectionTested.value = false
    testedClusterInfo.value = null
  },
  { immediate: true }
)

// Reset test state when form values change
watch(
  () => [formValue.value.bootstrap_servers, formValue.value.security_protocol, formValue.value.sasl_mechanism, formValue.value.sasl_username, formValue.value.sasl_password],
  () => {
    connectionTested.value = false
    testedClusterInfo.value = null
  }
)

const securityProtocols = [
  { label: 'PLAINTEXT', value: 'PLAINTEXT' },
  { label: 'SSL', value: 'SSL' },
  { label: 'SASL_PLAINTEXT', value: 'SASL_PLAINTEXT' },
  { label: 'SASL_SSL', value: 'SASL_SSL' },
]

const saslMechanisms = [
  { label: 'PLAIN', value: 'PLAIN' },
  { label: 'SCRAM-SHA-256', value: 'SCRAM_SHA_256' },
  { label: 'SCRAM-SHA-512', value: 'SCRAM_SHA_512' },
  { label: 'GSSAPI (Kerberos)', value: 'GSSAPI' },
]

const requiresSasl = computed(() =>
  ['SASL_PLAINTEXT', 'SASL_SSL'].includes(formValue.value.security_protocol || '')
)

const isEditing = computed(() => !!props.editCluster)

// 验证表单
const validateForm = () => {
  if (!formValue.value.name) {
    message.warning(t('cluster.nameRequired', 'Cluster name is required'))
    return false
  }
  if (!formValue.value.bootstrap_servers) {
    message.warning(t('cluster.bootstrapServersRequired', 'Bootstrap servers is required'))
    return false
  }
  // SASL 机制在 SASL_PLAINTEXT 和 SASL_SSL 时必填
  if (requiresSasl.value && !formValue.value.sasl_mechanism) {
    message.warning(t('cluster.saslMechanismRequired', 'SASL mechanism is required for SASL_PLAINTEXT and SASL_SSL'))
    return false
  }
  return true
}

const handleTestConnection = async () => {
  if (!validateForm()) return

  testingConnection.value = true
  connectionTested.value = false
  testedClusterInfo.value = null

  const config: ClusterConfig = {
    id: props.editCluster?.config.id || crypto.randomUUID(),
    name: formValue.value.name || 'test-connection',
    bootstrap_servers: formValue.value.bootstrap_servers || '',
    security_protocol: formValue.value.security_protocol || 'PLAINTEXT',
    sasl_mechanism: formValue.value.sasl_mechanism,
    sasl_username: formValue.value.sasl_username,
    sasl_password: formValue.value.sasl_password,
  }

  try {
    const info = await clusterStore.testClusterConnection(config)
    connectionTested.value = true
    testedClusterInfo.value = info
    message.success(t('cluster.testSuccess', 'Connection successful'))
  } catch (e: unknown) {
    connectionTested.value = false
    message.error(t('cluster.testFailed', 'Connection failed') + ': ' + String(e))
  } finally {
    testingConnection.value = false
  }
}

const handleConnect = async () => {
  if (!validateForm()) return

  const config: ClusterConfig = {
    id: props.editCluster?.config.id || crypto.randomUUID(),
    name: formValue.value.name || '',
    bootstrap_servers: formValue.value.bootstrap_servers || '',
    security_protocol: formValue.value.security_protocol || 'PLAINTEXT',
    sasl_mechanism: formValue.value.sasl_mechanism,
    sasl_username: formValue.value.sasl_username,
    sasl_password: formValue.value.sasl_password,
  }

  try {
    const clientId = await clusterStore.connectCluster(config)

    // 添加集群 Tab
    tabStore.addClusterTab({
      clusterId: config.id,
      clusterName: config.name,
      clientId,
      connected: true
    })

    message.success(props.editCluster ? t('cluster.reconnectSuccess', 'Cluster reconnected successfully') : t('cluster.connectSuccess', 'Cluster connected successfully'))
    emit('success')
    emit('update:show', false)
    resetForm()
  } catch (e: unknown) {
    message.error(t('cluster.connectFailed', 'Failed to connect') + ': ' + String(e))
  }
}

const handleSave = () => {
  if (!validateForm()) return

  const config: ClusterConfig = {
    id: props.editCluster?.config.id || crypto.randomUUID(),
    name: formValue.value.name || '',
    bootstrap_servers: formValue.value.bootstrap_servers || '',
    security_protocol: formValue.value.security_protocol || 'PLAINTEXT',
    sasl_mechanism: formValue.value.sasl_mechanism,
    sasl_username: formValue.value.sasl_username,
    sasl_password: formValue.value.sasl_password,
  }

  clusterStore.updateCluster(config)
  message.success(t('cluster.saveSuccess', 'Cluster configuration saved'))
  emit('success')
  emit('update:show', false)
  resetForm()
}

const resetForm = () => {
  formValue.value = defaultFormValue()
  connectionTested.value = false
  testedClusterInfo.value = null
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
          <component :is="isEditing ? Pencil : Plus" :size="18" :stroke-width="1.5" class="header-icon" />
          {{ isEditing ? t('cluster.editCluster', 'Edit Cluster') : t('cluster.addCluster', 'Add Cluster') }}
        </span>
        <X :size="16" class="header-close" @click="handleClose" />
      </div>

      <!-- Content -->
      <div class="modal-content">
        <!-- Cluster Name -->
        <div class="form-row">
          <label class="form-label">{{ t('cluster.name', 'Cluster Name') }}<span class="required">*</span></label>
          <n-input
            v-model:value="formValue.name"
            :placeholder="t('cluster.namePlaceholder', 'My Kafka Cluster')"
            size="small"
          />
        </div>

        <!-- Bootstrap Servers -->
        <div class="form-row">
          <label class="form-label">{{ t('cluster.bootstrapServers', 'Bootstrap Servers') }}<span class="required">*</span></label>
          <n-input
            v-model:value="formValue.bootstrap_servers"
            placeholder="localhost:9092"
            size="small"
          />
        </div>

        <!-- Security Protocol -->
        <div class="form-row">
          <label class="form-label">{{ t('cluster.securityProtocol', 'Security Protocol') }}</label>
          <n-select
            v-model:value="formValue.security_protocol"
            :options="securityProtocols"
            size="small"
          />
        </div>

        <!-- SASL Configuration -->
        <template v-if="requiresSasl">
          <div class="section-divider">
            <span class="divider-text">{{ t('cluster.saslConfig', 'SASL Configuration') }}</span>
          </div>

          <div class="form-row">
            <label class="form-label">{{ t('cluster.saslMechanism', 'SASL Mechanism') }}<span class="required">*</span></label>
            <n-select
              v-model:value="formValue.sasl_mechanism"
              :options="saslMechanisms"
              size="small"
            />
          </div>

          <div class="form-row">
            <label class="form-label">{{ t('cluster.username', 'Username') }}</label>
            <n-input
              v-model:value="formValue.sasl_username"
              size="small"
            />
          </div>

          <div class="form-row">
            <label class="form-label">{{ t('cluster.password', 'Password') }}</label>
            <n-input
              v-model:value="formValue.sasl_password"
              type="password"
              show-password-on="click"
              size="small"
            />
          </div>
        </template>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <div class="footer-left">
          <span class="footer-hint">{{ isEditing ? t('cluster.editClusterHint', 'Modify cluster configuration') : t('cluster.addClusterHint', 'Connect to a new cluster') }}</span>
          <div v-if="testedClusterInfo" class="test-result">
            <Check :size="14" class="test-success-icon" />
            <span>{{ testedClusterInfo.brokers.length }} {{ t('cluster.brokers', 'brokers') }}</span>
          </div>
        </div>
        <div class="footer-actions">
          <button class="action-btn test-btn" :class="{ 'tested': connectionTested }" :disabled="testingConnection" @click="handleTestConnection">
            <Zap :size="14" :class="{ 'spin': testingConnection }" />
            {{ testingConnection ? t('cluster.testing', 'Testing...') : (connectionTested ? t('cluster.tested', 'Tested') : t('cluster.testConnection', 'Test')) }}
          </button>
          <button class="action-btn" @click="handleClose">{{ t('common.cancel') }}</button>
          <template v-if="isEditing">
            <button class="action-btn" @click="handleSave">{{ t('cluster.save', 'Save') }}</button>
            <button
              class="action-btn primary"
              :disabled="clusterStore.loading"
              @click="handleConnect"
            >
              {{ t('cluster.reconnect', 'Reconnect') }}
            </button>
          </template>
          <button
            v-else
            class="action-btn primary"
            :disabled="clusterStore.loading"
            @click="handleConnect"
          >
            {{ t('cluster.connect', 'Connect') }}
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
  width: 520px;
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
  max-height: 60vh;
  overflow-y: auto;
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
  width: 120px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
  padding-top: 6px;
}

.required {
  color: var(--error);
  margin-left: 2px;
}

.section-divider {
  display: flex;
  align-items: center;
  margin: 20px 0 16px;
}

.section-divider::before,
.section-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border);
}

.divider-text {
  padding: 0 12px;
  font-size: 12px;
  color: var(--text-muted);
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--border);
}

.footer-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.footer-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.test-result {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--success);
}

.test-success-icon {
  color: var(--success);
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

.action-btn.test-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
}

.action-btn.test-btn:hover {
  background: var(--bg-hover);
  color: var(--accent);
}

.action-btn.test-btn.tested {
  color: var(--success);
  border: 1px solid var(--success);
}

.action-btn.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.action-btn.test-btn .spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

</style>
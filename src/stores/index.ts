import { createPinia } from 'pinia'

export const pinia = createPinia()

// Re-export stores
export { useClusterStore } from './cluster'
export { useTopicStore } from './topic'
export { useMessageStore } from './message'
export { useConsumerStore } from './consumer'
export { useTabStore } from './tabs'
<!-- src/views/tabs/SendTestTab.vue -->
<script setup lang="ts">
import { type Tab } from '@/stores/tabs'
import ProduceForm from '@/components/message/ProduceForm.vue'

defineProps<{
  tab: Tab
}>()
</script>

<template>
  <div class="send-test-tab">
    <ProduceForm :topic-name="tab.params.topicName" />
  </div>
</template>

<style scoped>
.send-test-tab {
  padding: 0;
}
</style>
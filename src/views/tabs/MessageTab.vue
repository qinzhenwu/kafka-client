<!-- src/views/tabs/MessageTab.vue -->
<script setup lang="ts">
import { type Tab } from '@/stores/tabs'
import MessageList from '@/components/message/MessageList.vue'

defineProps<{
  tab: Tab
}>()
</script>

<template>
  <div class="message-tab">
    <MessageList :topic-name="tab.params.topicName" />
  </div>
</template>

<style scoped>
.message-tab {
  padding: 0;
}
</style>
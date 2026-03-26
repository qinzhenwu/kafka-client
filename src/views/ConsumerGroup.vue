<script setup lang="ts">
import { ref } from 'vue'
import { NGrid, NGi } from 'naive-ui'
import ConsumerGroupList from '@/components/consumer/ConsumerGroupList.vue'
import ConsumerGroupDetail from '@/components/consumer/ConsumerGroupDetail.vue'

const selectedGroupId = ref<string | null>(null)

const handleSelect = (groupId: string) => {
  selectedGroupId.value = groupId
}

const handleBack = () => {
  selectedGroupId.value = null
}
</script>

<template>
  <div>
    <n-grid :cols="1" :x-gap="16">
      <n-gi>
        <ConsumerGroupList
          v-if="!selectedGroupId"
          @select="handleSelect"
        />
        <ConsumerGroupDetail
          v-else
          :group-id="selectedGroupId"
          @back="handleBack"
        />
      </n-gi>
    </n-grid>
  </div>
</template>
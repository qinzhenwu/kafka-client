<script setup lang="ts">
defineProps<{
  size?: number
  strokeWidth?: number
}>()
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    :width="size || 24"
    :height="size || 24"
    viewBox="0 0 15 17"
    fill="none"
  >
    <!-- 文档底座 -->
    <path
      d="M3 1.5C3 0.94772 3.44772 0.5 4 0.5H10.5L14 4V15.5C14 16.0523 13.5523 16.5 13 16.5H4C3.44772 16.5 3 16.0523 3 15.5V1.5Z"
      stroke="currentColor"
      stroke-width="1"
    />
    <!-- 折角 -->
    <path
      d="M10.5 0.5V3C10.5 3.55228 10.9477 4 11.5 4H14"
      stroke="currentColor"
      stroke-width="1"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- G 字母 -->
    <path
      d="M10.5 7C10 6.5 9 6 8 6C6.5 6 5.5 7 5.5 8.5C5.5 10 6.5 11 8 11C9.25 11 10.25 10 10.5 9M8 8.5H10.5"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
    />
    <!-- 底部成员点 -->
    <circle cx="5" cy="14.5" r="0.8" fill="currentColor" />
    <circle cx="8.5" cy="14.5" r="0.8" fill="currentColor" />
    <circle cx="12" cy="14.5" r="0.8" fill="currentColor" />
  </svg>
</template>
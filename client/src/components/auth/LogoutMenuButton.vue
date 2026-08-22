<script setup lang="ts">
defineProps<{
  displayName: string
  open: boolean
}>()
</script>

<template>
  <button type="button" class="px-4 py-2 bg-gray-100 text-gray-900 rounded-md">
    {{ displayName }} {{ open ? '▲' : '▼' }}
  </button>
</template>

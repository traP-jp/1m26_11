<script setup lang="ts">
defineProps<{
  displayName: string
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
}>()
</script>

<template>
  <button type="button" :aria-expanded="open" @click="emit('toggle')">
    {{ displayName }} {{ open ? '▲' : '▼' }}
  </button>
</template>

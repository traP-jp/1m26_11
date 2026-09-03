<script setup lang="ts">
withDefaults(
  defineProps<{
    displayName: string
    open: boolean
    disabled?: boolean
  }>(),
  { disabled: false },
)
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    class="rounded-md bg-gray-100 px-4 py-2 text-gray-900 disabled:cursor-not-allowed disabled:opacity-60"
  >
    {{ displayName }} {{ open ? '▲' : '▼' }}
  </button>
</template>

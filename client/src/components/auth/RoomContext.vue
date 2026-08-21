<script setup lang="ts">
defineProps({
  roomNumber: {
    type: String,
    required: true,
  },
  roomName: {
    type: String,
    required: true,
  },
})
</script>

<template>
  <div class="flex items-baseline space-x-3 px-4 py-2">
    <span class="text-xs font-bold text-gray-800 tracking-wider"> ROOM {{ roomNumber }} </span>
    <span class="text-base font-bold text-gray-800">
      {{ roomName }}
    </span>
  </div>
</template>

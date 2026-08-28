<script setup lang="ts">
import type { RoomUiEvent, RoomViewModel } from './RoomPage.types'
import RoomView from './RoomView.vue'

defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()
</script>

<template>
  <RoomView :view-model="viewModel" @ui-event="emit('uiEvent', $event)" />
</template>

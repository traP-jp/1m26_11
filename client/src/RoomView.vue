<script setup lang="ts">
import type { RoomUiEvent, RoomViewModel } from './RoomPage.types'

defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()
</script>

<template>
  <main>
    <h1>{{ viewModel.room.name }}</h1>
    <p>経過時間: {{ viewModel.serverElapsedMs }} ms</p>
    <button type="button" @click="emit('uiEvent', { type: 'room-exited' })">退出する</button>

    <section v-if="viewModel.selectedProblem" aria-labelledby="problem-title">
      <h2 id="problem-title">{{ viewModel.selectedProblem.title }}</h2>
      <p>{{ viewModel.selectedProblem.bodyMarkdown }}</p>
    </section>
  </main>
</template>

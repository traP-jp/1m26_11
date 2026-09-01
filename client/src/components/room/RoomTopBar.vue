<script setup lang="ts">
import RoomContext from './RoomContext.vue'
import GameTimer from './GameTimer.vue'
import ExitButton from './ExitButton.vue'

export interface RoomTopBarProps {
  roomNumber: string
  roomName: string
  serverElapsedMs: number
  active: boolean
}

defineProps<RoomTopBarProps>()

const emit = defineEmits(['exit'])

function handleExit() {
  emit('exit')
}
</script>

<template>
  <header class="flex items-center justify-between w-full px-6 py-4 bg-white shadow-md rounded-2xl">
    <div class="flex-1 flex justify-start items-center">
      <RoomContext
        :roomNumber="roomNumber"
        :roomName="roomName"
        class="transform scale-100 origin-left"
      />
    </div>

    <div class="flex-1 flex justify-center items-center">
      <GameTimer :serverElapsedMs="serverElapsedMs" :active="active" class="transform scale-100" />
    </div>

    <div class="flex-1 flex justify-end items-center">
      <ExitButton @exit="handleExit" class="transform scale-100 origin-right" />
    </div>
  </header>
</template>

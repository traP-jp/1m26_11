<script setup lang="ts">
import { RouterView, useRouter } from 'vue-router'

import { roomPageFixture } from './RoomPage.fixture'
import type { RoomUiEvent } from './RoomPage.types'

const router = useRouter()

function handleRoomSelected(roomId: string): void {
  void router.push({ name: 'room', params: { roomId } })
}

function handleRoomUiEvent(event: RoomUiEvent): void {
  if (event.type === 'room-exited') void router.push({ name: 'portal' })
}
</script>

<template>
  <RouterView v-slot="{ Component, route }">
    <component :is="Component" v-if="route.name === 'portal'" @room-selected="handleRoomSelected" />
    <component
      :is="Component"
      v-else-if="route.name === 'room'"
      :view-model="roomPageFixture"
      @ui-event="handleRoomUiEvent"
    />
    <component :is="Component" v-else />
  </RouterView>
</template>

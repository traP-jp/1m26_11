<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import ClearPage from './ClearPage.vue'
import PortalPage from './PortalPage.vue'
import RoomPage from './RoomPage.vue'
import { roomPageFixture } from './RoomPage.fixture'
import type { RoomUiEvent } from './RoomPage.types'
import { resolveRoute } from './routes'

const pathname = ref(window.location.pathname)
const route = computed(() => resolveRoute(pathname.value))

function syncPathname(): void {
  pathname.value = window.location.pathname
}

function navigate(path: string): void {
  window.history.pushState({}, '', path)
  syncPathname()
}

onMounted(() => window.addEventListener('popstate', syncPathname))
onBeforeUnmount(() => window.removeEventListener('popstate', syncPathname))

function handleRoomSelected(roomId: string): void {
  navigate(`/rooms/${encodeURIComponent(roomId)}`)
}

function handleRoomUiEvent(event: RoomUiEvent): void {
  if (event.type === 'room-exited') navigate('/')
}
</script>

<template>
  <PortalPage v-if="route.name === 'portal'" @room-selected="handleRoomSelected" />
  <RoomPage
    v-else-if="route.name === 'room'"
    :view-model="roomPageFixture"
    @ui-event="handleRoomUiEvent"
  />
  <ClearPage v-else-if="route.name === 'clear'" />
  <PortalPage v-else @room-selected="handleRoomSelected" />
</template>

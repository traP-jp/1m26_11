<script setup lang="ts">
import PortalHeader from './components/portal/PortalHeader.vue'
import { portalHeaderFixtures } from './components/portal/PortalHeader.fixture'
import RoomCard, { type Room } from './RoomCard.vue'

const emit = defineEmits<{
  roomSelected: [roomId: string]
}>()

const rooms: Room[] = [
  {
    room_id: '1411824c-d357-4941-af76-c76cb827dda6',
    number: 1,
    name: '最初の部屋',
    genre: 'logic',
    description: '動作確認用の問題セットです',
  },
  {
    room_id: '1411444c-d357-4941-af76-c76cb827dda6',
    number: 2,
    name: '2番目の部屋',
    genre: 'logic',
    description: '動作確認用の問題セットです',
  },
]
</script>

<template>
  <PortalHeader v-bind="portalHeaderFixtures.demoAuthenticated" />
  <main class="portal-page">
    <h1 class="portal-page__title">Portal</h1>
    <p class="portal-page__description">挑戦する部屋を選んでください。</p>
    <section aria-label="部屋一覧" class="card-list">
      <RoomCard
        v-for="room in rooms"
        :key="room.room_id"
        :room="room"
        @start="emit('roomSelected', $event)"
      />
    </section>
  </main>
</template>

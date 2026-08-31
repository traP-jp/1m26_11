<script setup lang="ts">
import { inject, onMounted, ref } from 'vue'

import GuestNameForm from './components/auth/GuestNameForm.vue'
import PortalHeader from './components/portal/PortalHeader.vue'
import RoomCard, { type Room } from './RoomCard.vue'
import { authApiClientKey, createAuthFlow } from './utils/auth'

const emit = defineEmits<{
  roomSelected: [roomId: string]
}>()

const auth = createAuthFlow(inject(authApiClientKey))
const guestNameForm = ref<InstanceType<typeof GuestNameForm> | null>(null)

function focusGuestNameForm(): void {
  guestNameForm.value?.focus()
}

onMounted(() => void auth.refresh())

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
  <p v-if="auth.state.value.status === 'loading'" role="status">認証状態を確認しています…</p>
  <p v-else-if="auth.state.value.status === 'error'" role="alert">
    認証状態を取得できませんでした。
    <button type="button" @click="auth.refresh">再試行</button>
  </p>
  <template v-else>
    <PortalHeader
      v-if="auth.portalUserStatus.value"
      home-href="/"
      instructions-href="#instructions"
      :user-status="auth.portalUserStatus.value"
      @login="focusGuestNameForm"
      @logout="auth.logout"
    />
    <p v-if="auth.state.value.error" role="alert">認証操作に失敗しました。再度お試しください。</p>
    <GuestNameForm
      v-if="auth.state.value.status === 'unauthenticated' && auth.state.value.authMode === 'demo'"
      ref="guestNameForm"
      :submit-pending="auth.state.value.busy"
      @submit="auth.loginGuest"
    />
  </template>
  <main v-if="auth.state.value.status === 'authenticated'" class="portal-page">
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

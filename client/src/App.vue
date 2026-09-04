<script setup lang="ts">
import { computed, inject, onMounted } from 'vue'
import { RouterView, useRouter } from 'vue-router'

import { portalPageFixtures } from './PortalPage.fixture'
import type { PortalPageProps } from './PortalPage.types'
import { roomPageFixture } from './RoomPage.fixture'
import type { RoomUiEvent } from './RoomPage.types'
import { authApiClientKey, authNavigationHandlerKey, createAuthController } from './utils/auth'

const router = useRouter()
const auth = createAuthController(inject(authApiClientKey), inject(authNavigationHandlerKey))

const portalPageProps = computed<PortalPageProps | null>(() => {
  const state = auth.state.value
  if (state.status === 'unauthenticated') {
    return {
      ...portalPageFixtures.demoUnauthenticated,
      authenticated: false,
      authMode: state.authMode,
      displayName: null,
      authBusy: state.busy,
      loginHref: state.loginUrl,
      logoutHref: null,
    }
  }
  if (state.status === 'authenticated') {
    return {
      ...portalPageFixtures.demoAuthenticated,
      authenticated: true,
      authMode: state.authMode,
      displayName: state.displayName,
      authBusy: state.busy,
      loginHref: null,
      logoutHref: state.logoutUrl,
    }
  }
  return null
})

const hasAuthOperationError = computed(() => {
  const state = auth.state.value
  return (
    (state.status === 'authenticated' || state.status === 'unauthenticated') && state.error !== null
  )
})

onMounted(() => void auth.refresh())

function handleLogin(): void {
  void auth.login()
}

function handleRoomSelected(roomId: string): void {
  void router.push({ name: 'room', params: { roomId } })
}

function handleProblemAuthoring(roomId: string): void {
  void router.push({
    name: 'problem-author-new',
    params: { roomId },
  })
}

function handleRoomUiEvent(event: RoomUiEvent): void {
  if (event.type === 'room-exited' || event.type === 'portal-returned') {
    void router.push({ name: 'portal' })
  }
}
</script>

<template>
  <RouterView v-slot="{ Component, route }">
    <template v-if="route.name === 'portal'">
      <p v-if="auth.state.value.status === 'loading'" role="status">認証状態を確認しています…</p>
      <p v-else-if="auth.state.value.status === 'error'" role="alert">
        認証状態を取得できませんでした。
        <button type="button" @click="auth.refresh">再試行</button>
      </p>
      <template v-else-if="portalPageProps">
        <p v-if="hasAuthOperationError" role="alert">
          認証操作に失敗しました。再度お試しください。
        </p>
        <component
          :is="Component"
          v-bind="portalPageProps"
          @login="handleLogin"
          @guest-login="auth.loginGuest"
          @logout="auth.logout"
          @start-room="handleRoomSelected"
          @author-room="handleProblemAuthoring"
        />
      </template>
    </template>
    <component
      :is="Component"
      v-else-if="route.name === 'room'"
      :view-model="roomPageFixture"
      @ui-event="handleRoomUiEvent"
    />
    <component
      :is="Component"
      v-else-if="route.name === 'problem-author-new'"
      :room-id="typeof route.params.roomId === 'string' ? route.params.roomId : ''"
    />
    <component :is="Component" v-else />
  </RouterView>
</template>

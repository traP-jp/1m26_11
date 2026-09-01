<script setup lang="ts">
import { computed } from 'vue'

import AuthActionButton from './components/auth/AuthActionButton.vue'
import GuestNameForm from './components/auth/GuestNameForm.vue'
import MinimalProgressSummary from './components/portal/MinimalProgressSummary.vue'
import PortalHeader from './components/portal/PortalHeader.vue'
import PortalLoginPrompt from './components/portal/PortalLoginPrompt.vue'
import type { PortalUserStatusState } from './components/portal/PortalHeader.types'
import type { PortalPageProps } from './PortalPage.types'
import RoomCard from './RoomCard.vue'

const props = defineProps<PortalPageProps>()
const emit = defineEmits<{
  login: []
  guestLogin: [displayName: string]
  logout: []
  showInstructions: []
  startRoom: [roomId: string]
}>()

const userStatus = computed<PortalUserStatusState>(() =>
  props.authenticated
    ? {
        authenticated: true,
        authMode: props.authMode,
        displayName: props.displayName ?? '',
        logoutHref: null,
        logoutPending: props.authBusy,
      }
    : {
        authenticated: false,
        authMode: props.authMode,
        loginHref: null,
        loginPending: props.authBusy,
      },
)
</script>

<template>
  <PortalHeader
    home-href="/"
    :user-status="userStatus"
    @login="emit('login')"
    @logout="emit('logout')"
    @show-instructions="emit('showInstructions')"
  />
  <main class="portal-page">
    <PortalLoginPrompt v-if="!authenticated">
      <template #action>
        <GuestNameForm
          v-if="authMode === 'demo'"
          :submit-pending="authBusy"
          @submit="emit('guestLogin', $event)"
        />
        <AuthActionButton
          v-else
          action="login"
          :disabled="authBusy"
          :label="authBusy ? '処理中…' : undefined"
          @activate="emit('login')"
        />
      </template>
    </PortalLoginPrompt>

    <template v-else>
      <h1 class="portal-page__title">Portal</h1>
      <p class="portal-page__description">挑戦する部屋を選んでください。</p>
      <MinimalProgressSummary :status="progressStatus" />
      <section aria-label="必須の部屋" class="card-list">
        <RoomCard :room="requiredRoom" :starting="authBusy" @start="emit('startRoom', $event)" />
      </section>
    </template>
  </main>
</template>

<script setup lang="ts">
import UserMenu from '../auth/UserMenu.vue'
import type { PortalUserStatusState } from './PortalHeader.types'

defineProps<{
  status: PortalUserStatusState
}>()

const emit = defineEmits<{
  login: []
  logout: []
}>()
</script>

<template>
  <div v-if="!status.authenticated" class="relative shrink-0">
    <a
      v-if="status.loginHref"
      class="inline-flex min-h-8 cursor-pointer items-center justify-center gap-[0.35rem] rounded-[0.45rem] border border-[#121a2a] bg-[#121a2a] px-3 py-2 text-[0.6875rem] font-extrabold leading-none text-white no-underline transition-[border-color,background-color,opacity] duration-150 hover:border-[#27354d] hover:bg-[#27354d] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea]"
      :href="status.loginHref"
    >
      ログイン
    </a>
    <button
      v-else
      class="inline-flex min-h-8 cursor-pointer items-center justify-center gap-[0.35rem] rounded-[0.45rem] border border-[#121a2a] bg-[#121a2a] px-3 py-2 text-[0.6875rem] font-extrabold leading-none text-white transition-[border-color,background-color,opacity] duration-150 hover:border-[#27354d] hover:bg-[#27354d] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea] disabled:cursor-not-allowed disabled:opacity-[0.58]"
      type="button"
      :disabled="status.loginPending"
      :aria-busy="status.loginPending"
      @click="emit('login')"
    >
      {{ status.loginPending ? '処理中…' : 'ログイン' }}
    </button>
  </div>

  <div v-else class="relative shrink-0" :data-auth-mode="status.authMode">
    <UserMenu
      :display-name="status.displayName"
      :logout-href="status.logoutHref"
      :logout-pending="status.logoutPending"
      @logout="emit('logout')"
    />
  </div>
</template>

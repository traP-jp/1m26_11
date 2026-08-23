<script setup lang="ts">
import PortalUserStatus from './PortalUserStatus.vue'
import type { PortalHeaderProps } from './PortalHeader.types'

withDefaults(defineProps<PortalHeaderProps>(), {
  instructionsHref: null,
})

const emit = defineEmits<{
  login: []
  logout: []
  showInstructions: []
}>()

const instructionControlClasses = [
  'inline-flex min-h-8 cursor-pointer items-center justify-center gap-[0.35rem]',
  'rounded-[0.45rem] border border-[#d8e1ee] bg-[#f5f8fc] px-3 py-[0.45rem]',
  'text-[0.6875rem] font-bold leading-none text-[#22324c]',
  'transition-colors duration-150 hover:border-[#bdcadd] hover:bg-[#edf3fa]',
  'focus-visible:outline-2 focus-visible:outline-offset-[3px]',
  'focus-visible:outline-[#3997ea] max-[34rem]:w-8 max-[34rem]:px-0',
]
</script>

<template>
  <header
    class="relative flex h-18 w-full items-center justify-between overflow-visible border-b border-[#e5eaf2] bg-white px-8 py-3 text-[#121a2a] max-[34rem]:h-16 max-[34rem]:px-4"
  >
    <a
      class="inline-flex shrink-0 flex-col leading-none text-inherit no-underline focus-visible:outline-2 focus-visible:outline-offset-[3px] focus-visible:outline-[#3997ea]"
      :href="homeHref"
      aria-label="ワンマンそん ホーム"
    >
      <span class="text-[0.9375rem] font-extrabold tracking-[0.025em]">ワンマンそん</span>
    </a>

    <nav class="flex items-center gap-2.5 max-[34rem]:gap-[0.45rem]" aria-label="ポータル操作">
      <a
        v-if="instructionsHref"
        class="no-underline"
        :class="instructionControlClasses"
        :href="instructionsHref"
        aria-label="操作説明"
      >
        <svg
          class="h-3.5 w-3.5 fill-none stroke-current stroke-[1.5] [stroke-linecap:round] [stroke-linejoin:round]"
          aria-hidden="true"
          viewBox="0 0 16 16"
        >
          <path
            d="M6.6 6a1.45 1.45 0 1 1 2.15 1.27C8.25 7.56 8 7.91 8 8.5v.25M8 11.25h.01M8 14.25A6.25 6.25 0 1 0 8 1.75a6.25 6.25 0 0 0 0 12.5Z"
          />
        </svg>
        <span class="max-[34rem]:sr-only">操作説明</span>
      </a>
      <button
        v-else
        :class="instructionControlClasses"
        type="button"
        aria-label="操作説明"
        @click="emit('showInstructions')"
      >
        <svg
          class="h-3.5 w-3.5 fill-none stroke-current stroke-[1.5] [stroke-linecap:round] [stroke-linejoin:round]"
          aria-hidden="true"
          viewBox="0 0 16 16"
        >
          <path
            d="M6.6 6a1.45 1.45 0 1 1 2.15 1.27C8.25 7.56 8 7.91 8 8.5v.25M8 11.25h.01M8 14.25A6.25 6.25 0 1 0 8 1.75a6.25 6.25 0 0 0 0 12.5Z"
          />
        </svg>
        <span class="max-[34rem]:sr-only">操作説明</span>
      </button>

      <PortalUserStatus :status="userStatus" @login="emit('login')" @logout="emit('logout')" />
    </nav>
  </header>
</template>

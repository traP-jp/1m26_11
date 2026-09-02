<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    action: 'login' | 'logout'
    label?: string
    disabled?: boolean
    href?: string | null
  }>(),
  {
    label: undefined,
    disabled: false,
    href: null,
  },
)

const emit = defineEmits<{
  activate: []
}>()

function activate(): void {
  if (!props.disabled) emit('activate')
}
</script>

<template>
  <a
    v-if="href && !disabled"
    :href="href"
    :data-action="action"
    class="inline-flex min-h-11 items-center justify-center rounded-lg bg-[#121a2a] px-5 py-3 font-bold text-white no-underline transition-colors hover:bg-[#27354d] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea]"
  >
    {{ label ?? (action === 'login' ? 'ログイン' : 'ログアウト') }}
  </a>
  <button
    v-else
    type="button"
    :disabled="disabled"
    :data-action="action"
    class="inline-flex min-h-11 items-center justify-center rounded-lg bg-[#121a2a] px-5 py-3 font-bold text-white transition-colors hover:bg-[#27354d] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea] disabled:cursor-not-allowed disabled:opacity-60"
    @click="activate"
  >
    {{ label ?? (action === 'login' ? 'ログイン' : 'ログアウト') }}
  </button>
</template>

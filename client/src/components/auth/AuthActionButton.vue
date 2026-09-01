<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    action: 'login' | 'logout'
    label?: string
    disabled?: boolean
  }>(),
  {
    label: undefined,
    disabled: false,
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
  <button
    type="button"
    :disabled="disabled"
    :data-action="action"
    class="inline-flex min-h-11 items-center justify-center rounded-lg bg-[#121a2a] px-5 py-3 font-bold text-white transition-colors hover:bg-[#27354d] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea] disabled:cursor-not-allowed disabled:opacity-60"
    @click="activate"
  >
    {{ label ?? (action === 'login' ? 'ログイン' : 'ログアウト') }}
  </button>
</template>

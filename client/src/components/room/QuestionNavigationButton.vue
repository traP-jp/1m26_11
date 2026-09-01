<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  direction: 'previous' | 'next'
  problemId: string
  disabled: boolean
}>()

const emit = defineEmits<{
  select: [problemId: string]
}>()

const label = computed(() => (props.direction === 'previous' ? '前の問題' : '次の問題'))

function selectProblem(): void {
  if (props.disabled) return
  emit('select', props.problemId)
}
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    :aria-label="`${label}へ移動`"
    class="inline-flex min-h-10 items-center justify-center rounded-lg border border-[#c8d5e8] bg-white px-4 text-sm font-bold text-[#36465f] transition-colors hover:border-[#8da9d2] hover:bg-[#eef4ff] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:border-[#d7e1ef] disabled:bg-[#f1f4f8] disabled:text-[#8a97aa]"
    @click="selectProblem"
  >
    {{ direction === 'previous' ? '←' : '→' }}
    <span :class="direction === 'previous' ? 'ml-2' : 'mr-2 order-first'">{{ label }}</span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

export interface Problem {
  problem_id: string
  room_id: string
  number: number
  problem_type: 'small' | 'final'
  title: string
  body_markdown: string
}

const props = defineProps<{
  problem: Problem
}>()

const TOTAL_SMALL_PROBLEMS = 3

const isFinal = computed(() => {
  return props.problem.problem_type === 'final'
})
</script>

<template>
  <div class="flex items-start w-full px-8 py-6 bg-white border-y border-gray-200">
    <div class="flex flex-col items-center justify-center shrink-0 w-24 h-24 bg-blue-50 rounded-xl">
      <span class="mb-1 text-xs font-bold tracking-widest text-blue-600"> QUESTION </span>
      <span class="text-3xl font-bold text-gray-900"> 謎 {{ problem.number }} </span>
    </div>

    <div class="flex flex-col flex-1 px-8 mt-1">
      <h2 class="mb-2 text-xl font-bold text-gray-900">
        {{ problem.title }}
      </h2>
      <p class="text-sm leading-relaxed text-gray-600 whitespace-pre-wrap">
        {{ problem.body_markdown }}
      </p>
    </div>

    <div class="shrink-0 mt-1">
      <div v-if="!isFinal" class="text-sm font-bold text-blue-600">
        小なぞ {{ problem.number }} / {{ TOTAL_SMALL_PROBLEMS }}
      </div>
      <div v-else class="text-sm font-bold text-purple-600">大なぞ</div>
    </div>
  </div>
</template>

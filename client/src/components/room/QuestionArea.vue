<script setup lang="ts">
import ProblemBodyAssets from './ProblemBodyAssets.vue'
import type { ProblemBodyAssetsProps } from './ProblemBodyAssets.types'
import QuestionHeader from './QuestionHeader.vue'

export interface Props {
  problemNumber: number
  title: string
  problemType: 'small' | 'final'
  smallIndex?: number
  smallTotal?: number
  bodyMarkdown: ProblemBodyAssetsProps['bodyMarkdown']
  assets: ProblemBodyAssetsProps['assets']
}

defineProps<Props>()
</script>

<template>
  <section
    class="flex min-h-0 flex-col overflow-hidden rounded-2xl border border-[#d7e1ef] bg-white shadow-sm"
  >
    <QuestionHeader
      :problem-number="problemNumber"
      :title="title"
      :problem-type="problemType"
      :small-index="smallIndex"
      :small-total="smallTotal"
    />

    <div class="min-h-0 overflow-y-auto p-6 sm:p-8">
      <ProblemBodyAssets :body-markdown="bodyMarkdown" :assets="assets" />
    </div>
  </section>
</template>

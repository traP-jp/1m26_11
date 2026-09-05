<script setup lang="ts">
import { useId } from 'vue'

export type ProgressStatus = 'not_started' | 'active' | 'cleared'

defineProps<{
  status: ProgressStatus
}>()

const labels: Record<ProgressStatus, string> = {
  not_started: '未開始',
  active: '挑戦中',
  cleared: 'クリア済み',
}

const titleId = useId()
</script>

<template>
  <section class="rounded-xl border border-[#d8e1ee] bg-white p-5" :aria-labelledby="titleId">
    <h2 :id="titleId" class="text-sm font-bold text-[#65758d]">進捗</h2>
    <p class="mt-2 text-xl font-bold text-[#121a2a]" :data-status="status">{{ labels[status] }}</p>
  </section>
</template>

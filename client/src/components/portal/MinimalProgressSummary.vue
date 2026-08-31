<script setup lang="ts">
export type ProgressStatus = 'not_started' | 'in_progress' | 'cleared'

defineProps<{
  status: ProgressStatus
}>()

const labels: Record<ProgressStatus, string> = {
  not_started: '未開始',
  in_progress: '挑戦中',
  cleared: 'クリア済み',
}
</script>

<template>
  <section class="rounded-xl border border-[#d8e1ee] bg-white p-5" aria-labelledby="progress-title">
    <h2 id="progress-title" class="text-sm font-bold text-[#65758d]">進捗</h2>
    <p class="mt-2 text-xl font-bold text-[#121a2a]" :data-status="status">{{ labels[status] }}</p>
  </section>
</template>

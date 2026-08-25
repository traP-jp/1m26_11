<script setup lang="ts">
import { computed, useId } from 'vue'

type JudgeState = 'idle' | 'pending' | 'correct' | 'incorrect' | 'error'

interface AnswerJudgeResultProps {
  state: JudgeState
}

const props = defineProps<AnswerJudgeResultProps>()
const titleId = useId()

const presentationByState = {
  idle: {
    label: '判定待ち',
    description: '回答すると結果が表示されます',
    symbol: '—',
    badgeClasses: 'bg-[#eef3fa] text-[#52627a]',
  },
  pending: {
    label: '判定中',
    description: '回答を判定しています',
    symbol: '…',
    badgeClasses: 'bg-[#e7f0ff] text-[#2463d4]',
  },
  correct: {
    label: '正解',
    description: '回答は正解です',
    symbol: '○',
    badgeClasses: 'bg-[#ddf8e8] text-[#159447]',
  },
  incorrect: {
    label: '不正解',
    description: '回答は不正解です',
    symbol: '×',
    badgeClasses: 'bg-[#ffebec] text-[#d63844]',
  },
  error: {
    label: '判定エラー',
    description: '判定結果を取得できませんでした',
    symbol: '!',
    badgeClasses: 'bg-[#fff1da] text-[#a85c00]',
  },
} satisfies Record<
  JudgeState,
  {
    label: string
    description: string
    symbol: string
    badgeClasses: string
  }
>

const presentation = computed(() => presentationByState[props.state])
</script>

<template>
  <section
    class="flex min-h-72 w-full flex-col items-center rounded-xl border border-[#cbd8e9] bg-white px-6 py-5 text-[#152238] shadow-sm"
    :aria-labelledby="titleId"
    aria-live="polite"
    aria-atomic="true"
    :aria-busy="state === 'pending' ? 'true' : undefined"
    :data-state="state"
  >
    <h2 :id="titleId" class="text-sm font-bold">判定結果</h2>

    <div
      class="mt-4 flex h-24 w-24 items-center justify-center rounded-full"
      :class="presentation.badgeClasses"
      data-testid="result-badge"
    >
      <span class="text-5xl font-bold leading-none" aria-hidden="true" data-testid="result-symbol">
        {{ presentation.symbol }}
      </span>
    </div>

    <p class="mt-3 text-base font-bold" data-testid="result-label">
      {{ presentation.label }}
    </p>
    <p class="mt-1 text-center text-xs text-[#718099]">
      {{ presentation.description }}
    </p>
  </section>
</template>

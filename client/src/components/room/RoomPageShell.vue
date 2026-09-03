<script setup lang="ts">
import type { AlternativeInputSource } from '@/input/InputAdapter.types'

import type { RoomUiEvent, RoomViewModel } from '../../RoomPage.types'
import AnswerJudgeResult from './AnswerJudgeResult.vue'
import AnswerPanel from './AnswerPanel.vue'
import QuestionArea from './QuestionArea.vue'
import RoomTopBar from './RoomTopBar.vue'

defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()

function submitAnswer(answer: string, source: AlternativeInputSource): void {
  emit('uiEvent', { type: 'answer-submitted', source, answer })
}
</script>

<template>
  <main class="min-h-screen bg-[#eef3fa] p-4 text-[#121a2a] sm:p-6">
    <div
      class="mx-auto flex min-h-[calc(100vh-2rem)] max-w-7xl flex-col gap-4 sm:min-h-[calc(100vh-3rem)]"
    >
      <RoomTopBar
        :room-number="String(viewModel.room.number)"
        :room-name="viewModel.room.name"
        :server-elapsed-ms="viewModel.serverElapsedMs"
        active
        @exit="emit('uiEvent', { type: 'room-exited' })"
      />

      <QuestionArea
        v-if="viewModel.selectedProblem"
        class="min-h-80 flex-1"
        :problem-number="viewModel.selectedProblem.number"
        :title="viewModel.selectedProblem.title"
        :problem-type="viewModel.selectedProblem.type"
        :small-index="
          viewModel.selectedProblem.type === 'small' ? viewModel.selectedProblem.number : undefined
        "
        :small-total="viewModel.clear.requiredCount"
        :body-markdown="viewModel.selectedProblem.bodyMarkdown"
        :assets="viewModel.selectedProblem.assets"
      />
      <section
        v-else
        class="grid min-h-80 flex-1 place-items-center rounded-2xl border border-[#d7e1ef] bg-white p-6 text-sm font-bold text-[#65758d] shadow-sm"
        aria-label="問題表示"
      >
        表示できる問題がありません。
      </section>

      <div class="grid shrink-0 gap-4 lg:grid-cols-[minmax(0,7fr)_minmax(18rem,3fr)]">
        <AnswerPanel
          :max-length="viewModel.answerInput.maxLength"
          :pending="viewModel.answerJudgement.state === 'pending'"
          :disabled="viewModel.selectedProblem === null"
          @submit="submitAnswer"
        />
        <AnswerJudgeResult :state="viewModel.answerJudgement.state" />
      </div>
    </div>
  </main>
</template>

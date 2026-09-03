<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import {
  screenButtonInputSource,
  type AlternativeInputSource,
  type Control,
} from '@/input/InputAdapter.types'

import type { RoomUiEvent, RoomViewModel } from '../../RoomPage.types'
import AnswerJudgeResult from './AnswerJudgeResult.vue'
import AnswerPanel from './AnswerPanel.vue'
import ConditionBoxList from './ConditionBoxList.vue'
import type { ConditionBoxItem } from './ConditionBoxList.types'
import QuestionArea from './QuestionArea.vue'
import QuestionNavigationButton from './QuestionNavigationButton.vue'
import RoomTopBar from './RoomTopBar.vue'

const props = defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()

const controlLabels: Readonly<Record<Control, string>> = {
  up: '上',
  down: '下',
  left: '左',
  right: '右',
  red: '赤',
  yellow: '黄',
  green: '緑',
}

const selectedProblemIndex = computed(() =>
  props.viewModel.problems.findIndex((problem) => problem.selected),
)
const previousProblem = computed(() =>
  selectedProblemIndex.value > 0
    ? props.viewModel.problems[selectedProblemIndex.value - 1]
    : undefined,
)
const nextProblem = computed(() => {
  if (selectedProblemIndex.value < 0) return undefined
  return props.viewModel.problems[selectedProblemIndex.value + 1]
})

const conditionItems = computed<ConditionBoxItem[]>(() =>
  props.viewModel.queryInput.operations.map(({ control, count }, index) => ({
    id: `operation-${index}`,
    label: `${controlLabels[control]}を${count}回`,
  })),
)
const selectedControl = ref<Control | undefined>(props.viewModel.queryInput.allowedControls[0])
const queryDisabled = computed(
  () =>
    props.viewModel.selectedProblem === null || props.viewModel.queryJudgement.state === 'pending',
)

watch(
  () => props.viewModel.queryInput.allowedControls,
  (allowedControls) => {
    if (selectedControl.value === undefined || !allowedControls.includes(selectedControl.value)) {
      selectedControl.value = allowedControls[0]
    }
  },
)

function selectProblem(problemId: string): void {
  emit('uiEvent', { type: 'problem-selected', problemId })
}

function addCondition(): void {
  if (
    queryDisabled.value ||
    selectedControl.value === undefined ||
    props.viewModel.queryInput.operations.length >= props.viewModel.queryInput.maxOperations
  ) {
    return
  }

  emit('uiEvent', {
    type: 'condition-changed',
    source: screenButtonInputSource,
    control: selectedControl.value,
    count: 1,
  })
}

function removeCondition(itemId: string): void {
  if (queryDisabled.value) return
  const index = Number(itemId.replace('operation-', ''))
  const operation = props.viewModel.queryInput.operations[index]
  if (operation === undefined || itemId !== `operation-${index}`) return

  emit('uiEvent', {
    type: 'condition-changed',
    source: screenButtonInputSource,
    control: operation.control,
    count: -operation.count,
  })
}

function clearConditions(): void {
  if (queryDisabled.value) return
  for (const operation of props.viewModel.queryInput.operations) {
    emit('uiEvent', {
      type: 'condition-changed',
      source: screenButtonInputSource,
      control: operation.control,
      count: -operation.count,
    })
  }
}

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
        :small-total="Math.max(0, viewModel.clear.requiredCount - 1)"
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

      <nav class="flex items-center justify-between gap-4" aria-label="問題の移動">
        <QuestionNavigationButton
          direction="previous"
          :problem-id="previousProblem?.id ?? ''"
          :disabled="previousProblem === undefined || previousProblem.status === 'locked'"
          @select="selectProblem"
        />
        <QuestionNavigationButton
          direction="next"
          :problem-id="nextProblem?.id ?? ''"
          :disabled="nextProblem === undefined || nextProblem.status === 'locked'"
          @select="selectProblem"
        />
      </nav>

      <div class="grid gap-3 sm:grid-cols-[minmax(10rem,14rem)_minmax(0,1fr)] sm:items-start">
        <label class="text-sm font-bold text-[#36465f]">
          追加する操作
          <select
            v-model="selectedControl"
            class="mt-1 block min-h-10 w-full rounded-lg border border-[#c8d5e8] bg-white px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-[#2864e8]"
            :disabled="queryDisabled || viewModel.queryInput.allowedControls.length === 0"
          >
            <option
              v-for="control in viewModel.queryInput.allowedControls"
              :key="control"
              :value="control"
            >
              {{ controlLabels[control] }}
            </option>
          </select>
        </label>
        <ConditionBoxList
          :items="conditionItems"
          :disabled="queryDisabled"
          @add="addCondition"
          @remove="removeCondition"
          @clear="clearConditions"
        />
      </div>

      <div class="grid shrink-0 gap-4 lg:grid-cols-[minmax(0,7fr)_minmax(18rem,3fr)]">
        <AnswerPanel
          :max-length="viewModel.answerInput.maxLength"
          :pending="viewModel.answerJudgement.state === 'pending'"
          :disabled="
            viewModel.selectedProblem === null || viewModel.queryJudgement.state === 'pending'
          "
          @submit="submitAnswer"
        />
        <AnswerJudgeResult :state="viewModel.answerJudgement.state" />
      </div>
    </div>
  </main>
</template>

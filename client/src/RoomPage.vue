<script setup lang="ts">
import { computed, onBeforeUnmount, ref, useId, watch } from 'vue'

import SerialStatusNotice from './components/room/SerialStatusNotice.vue'
import type { SerialStatus } from './components/room/SerialStatusNotice.types'
import {
  type Control,
  type InputAdapterDispatcher,
  type InputAdapterEvent,
} from './input/InputAdapter.types'
import { createGuardedInputDispatcher } from './input/inputGuard'
import { createKeyboardInputAdapter } from './input/keyboardInputAdapter'
import { createScreenButtonInputAdapter } from './input/screenButtonInputAdapter'
import { useWebSerialConnection, type SerialConnectionState } from './input/useWebSerialConnection'
import { createWebSerialInputAdapter } from './input/webSerialInputAdapter'
import type { RoomUiEvent, RoomViewModel } from './RoomPage.types'
import ClearScreen from './components/room/ClearScreen.vue'
import RoomPageShell from './components/room/RoomPageShell.vue'

type InputMode = 'serial' | 'keyboard' | 'screen'

const props = defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()

const inputMode = ref<InputMode>('serial')
const screenAnswer = ref(props.viewModel.answerInput.value)
const inputError = ref<string | null>(null)
const selectedProblemStatus = computed(
  () => props.viewModel.problems.find((problem) => problem.selected)?.status,
)
const isOperationProblem = computed(
  () => props.viewModel.selectedProblem?.submissionType === 'operation_sequence',
)
const operationCount = computed(() =>
  props.viewModel.queryInput.operations.reduce((total, operation) => total + operation.count, 0),
)
const inputDisabled = computed(
  () =>
    props.viewModel.selectedProblem === null ||
    selectedProblemStatus.value !== 'available' ||
    props.viewModel.clear.cleared,
)
const roomDispatcher = createGuardedInputDispatcher(
  (event: InputAdapterEvent) => {
    inputError.value = null
    emit('uiEvent', event)
    return Promise.resolve()
  },
  {
    isDisabled: () => inputDisabled.value,
    isBusy: () =>
      props.viewModel.queryJudgement.state === 'pending' ||
      props.viewModel.answerJudgement.state === 'pending',
    onError(error) {
      inputError.value = error instanceof Error ? error.message : '入力処理に失敗しました。'
    },
  },
)
const inputUnavailable = computed(() => inputDisabled.value || roomDispatcher.busy)
const controlInputUnavailable = computed(
  () =>
    inputUnavailable.value ||
    !isOperationProblem.value ||
    operationCount.value >= props.viewModel.queryInput.maxOperations,
)
const querySubmissionUnavailable = computed(
  () =>
    inputUnavailable.value ||
    !isOperationProblem.value ||
    props.viewModel.queryInput.operations.length === 0,
)
const answerSubmissionUnavailable = computed(
  () => inputUnavailable.value || props.viewModel.selectedProblem?.submissionType !== 'string',
)

function createModeDispatcher(mode: InputMode): InputAdapterDispatcher {
  return {
    get busy() {
      return roomDispatcher.busy
    },
    dispatch(event) {
      if (inputMode.value !== mode) return false
      return roomDispatcher.dispatch(event)
    },
  }
}

function isControlAllowed(control: Control): boolean {
  return (
    isOperationProblem.value &&
    operationCount.value < props.viewModel.queryInput.maxOperations &&
    props.viewModel.queryInput.allowedControls.includes(control)
  )
}

const keyboardInput = createKeyboardInputAdapter({
  dispatcher: createModeDispatcher('keyboard'),
  isControlAllowed,
})
const screenInput = createScreenButtonInputAdapter({
  dispatcher: createModeDispatcher('screen'),
  isControlAllowed,
})
const serialInput = createWebSerialInputAdapter({
  dispatcher: createModeDispatcher('serial'),
  isControlAllowed,
})
const serialConnection = useWebSerialConnection({ adapter: serialInput })

watch(
  () => props.viewModel.clear.cleared,
  (cleared) => {
    if (!cleared) return
    keyboardInput.stop()
    void serialConnection.disconnect()
  },
  { immediate: true },
)

watch([() => props.viewModel.room.id, () => props.viewModel.selectedProblem?.id], () => {
  screenAnswer.value = props.viewModel.answerInput.value
})

function toNoticeStatus(state: SerialConnectionState): SerialStatus {
  switch (state.phase) {
    case 'unsupported':
      return 'unsupported'
    case 'requesting':
      return 'connecting'
    case 'connected':
      return 'connected'
    case 'idle':
    case 'disconnected':
      return 'disconnected'
    case 'error':
      if (state.operation === 'request-port') return 'denied'
      if (state.operation === 'reconnect') return 'retry-failed'
      return 'disconnected'
  }
}

const serialStatus = computed(() => toNoticeStatus(serialConnection.state.value))
const serialRetryLabel = computed(() => {
  const state = serialConnection.state.value
  return state.phase === 'error' && state.operation === 'close-port'
    ? 'Serialを解放する'
    : undefined
})
const inputModeLabelId = useId()

const keyboardKeyLabelByControl: Readonly<Record<string, string>> = {
  up: '↑（上）',
  down: '↓（下）',
  left: '←（左）',
  right: '→（右）',
  red: 'R（赤）',
  yellow: 'Y（黄）',
  green: 'G（緑）',
}

const keyboardInstructions = computed(() => {
  const keys = props.viewModel.queryInput.allowedControls.flatMap((control) => {
    const label = keyboardKeyLabelByControl[control]
    return label ? [label] : []
  })

  return keys.length > 0
    ? `${keys.join('、')}キーで操作できます。`
    : 'この問題で利用できるキーボード操作はありません。'
})

async function retrySerial(): Promise<void> {
  if (serialConnection.busy.value) return

  if (serialConnection.canDisconnect.value) {
    await serialConnection.disconnect()
    return
  }

  keyboardInput.stop()
  inputMode.value = 'serial'
  if (serialConnection.canConnect.value) {
    await serialConnection.connect()
  } else if (serialConnection.canRetry.value) {
    await serialConnection.retry()
  }
}

function useKeyboard(): void {
  keyboardInput.stop()
  inputMode.value = 'keyboard'
  keyboardInput.start()
  void serialConnection.disconnect()
}

function useScreen(): void {
  keyboardInput.stop()
  inputMode.value = 'screen'
  void serialConnection.disconnect()
}

function pressScreenControl(control: Control): void {
  screenInput.pressControl(control)
}

function submitScreenQuery(): void {
  screenInput.submitQuery()
}

function submitScreenAnswer(): void {
  screenInput.submitAnswer(screenAnswer.value)
}

const controlLabels: Readonly<Record<string, string>> = {
  up: '上',
  down: '下',
  left: '左',
  right: '右',
  red: '赤',
  yellow: '黄',
  green: '緑',
}

function controlLabel(control: string): string {
  return controlLabels[control] ?? control
}

onBeforeUnmount(() => keyboardInput.stop())
</script>

<template>
  <template v-if="!viewModel.clear.cleared">
    <aside class="mx-auto w-full max-w-6xl px-4 pt-4" aria-label="入力方法">
      <SerialStatusNotice
        :status="serialStatus"
        :retry-label="serialRetryLabel"
        :retry-disabled="serialConnection.busy.value"
        @retry="retrySerial"
        @use-keyboard="useKeyboard"
        @use-screen="useScreen"
      />

      <section
        v-if="inputMode !== 'serial'"
        class="mt-3 rounded-xl border border-[#c8d5e8] bg-white p-4 text-[#121a2a] shadow-sm"
        :data-input-mode="inputMode"
        :aria-labelledby="inputModeLabelId"
        aria-live="polite"
      >
        <p :id="inputModeLabelId" class="text-sm font-extrabold">
          {{ inputMode === 'keyboard' ? 'キーボード入力' : '画面ボタン入力' }}
        </p>
        <p v-if="inputMode === 'keyboard'" class="mt-1 text-sm text-[#52627a]">
          {{ keyboardInstructions }}
        </p>
        <div v-else class="mt-3 space-y-4">
          <div
            v-if="isOperationProblem"
            class="flex flex-wrap gap-2"
            role="group"
            :aria-labelledby="inputModeLabelId"
          >
            <button
              v-for="control in viewModel.queryInput.allowedControls"
              :key="control"
              type="button"
              class="min-h-10 min-w-16 rounded-lg bg-[#2864e8] px-4 text-sm font-bold text-white transition-colors hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:bg-[#a7b8d8]"
              :data-control="control"
              :disabled="controlInputUnavailable"
              @click="pressScreenControl(control)"
            >
              {{ controlLabel(control) }}
            </button>
          </div>

          <button
            v-if="isOperationProblem"
            type="button"
            data-testid="screen-submit-query"
            class="min-h-10 rounded-lg border border-[#2864e8] bg-white px-4 text-sm font-bold text-[#1f56cc] hover:bg-[#eef4ff] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:border-[#a7b8d8] disabled:text-[#78869a]"
            :disabled="querySubmissionUnavailable"
            @click="submitScreenQuery"
          >
            操作列を送信
          </button>

          <form
            v-if="viewModel.selectedProblem?.submissionType === 'string'"
            class="space-y-2"
            @submit.prevent="submitScreenAnswer"
          >
            <label class="block text-sm font-bold">
              文字列回答
              <textarea
                v-model="screenAnswer"
                data-testid="screen-answer-input"
                rows="2"
                :maxlength="viewModel.answerInput.maxLength"
                :disabled="answerSubmissionUnavailable"
                class="mt-1 block w-full rounded-lg border border-[#7aa7ff] bg-[#fbfcff] px-3 py-2 text-sm outline-none focus:border-[#2e6bea] focus:ring-2 focus:ring-[#2e6bea]/20 disabled:cursor-not-allowed disabled:bg-[#f1f4f8]"
              />
            </label>
            <button
              type="submit"
              data-testid="screen-submit-answer"
              class="min-h-10 rounded-lg bg-[#2864e8] px-4 text-sm font-bold text-white hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:bg-[#a7b8d8]"
              :disabled="answerSubmissionUnavailable"
            >
              回答を送信
            </button>
          </form>
        </div>

        <p v-if="inputError" class="mt-3 text-sm font-bold text-red-700" role="alert">
          {{ inputError }}
        </p>
      </section>
    </aside>
    <RoomPageShell
      :key="`${viewModel.room.id}:${viewModel.selectedProblem?.id ?? 'no-problem'}`"
      :view-model="viewModel"
      @ui-event="emit('uiEvent', $event)"
    />
  </template>
  <ClearScreen
    v-else
    :final-elapsed-ms="viewModel.serverElapsedMs"
    @back-to-portal="emit('uiEvent', { type: 'portal-returned' })"
  />
</template>

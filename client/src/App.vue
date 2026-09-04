<script setup lang="ts">
import { computed, inject, onMounted, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import { ApiClientError, apiClient, type GetRoomResponse } from './api/client'
import { QueryAnswerController } from './controllers/QueryAnswerController'
import { RunProblemController } from './controllers/RunProblemController'
import type { Operation } from './input/InputAdapter.types'
import { createOperationBuffer } from './input/operationBuffer'
import { portalPageFixtures } from './PortalPage.fixture'
import type { PortalPageProps } from './PortalPage.types'
import { roomPageFixture } from './RoomPage.fixture'
import type { JudgementState, RoomUiEvent, RoomViewModel } from './RoomPage.types'
import { authApiClientKey, authNavigationHandlerKey, createAuthController } from './utils/auth'

const router = useRouter()
const route = useRoute()
const client = inject(authApiClientKey) ?? apiClient
const auth = createAuthController(client, inject(authNavigationHandlerKey))
const operationBuffer = createOperationBuffer()
const queryAnswer = new QueryAnswerController(client, operationBuffer)
const runProblem = new RunProblemController(client, queryAnswer)

const roomLoading = ref(false)
const roomLoadError = ref<unknown | null>(null)
const roomDetails = ref<GetRoomResponse | null>(null)
const bufferedOperations = ref<Operation[]>([])
const lastSubmission = ref<'query' | 'answer' | null>(null)
let roomLoadGeneration = 0
let roomStartRequestedFor: string | null = null

const initialProblemId = roomPageFixture.selectedProblem?.id ?? null

const portalPageProps = computed<PortalPageProps | null>(() => {
  const state = auth.state.value
  if (state.status === 'unauthenticated') {
    return {
      ...portalPageFixtures.demoUnauthenticated,
      authenticated: false,
      authMode: state.authMode,
      displayName: null,
      authBusy: state.busy,
      loginHref: state.loginUrl,
      logoutHref: null,
    }
  }
  if (state.status === 'authenticated') {
    return {
      ...portalPageFixtures.demoAuthenticated,
      authenticated: true,
      authMode: state.authMode,
      displayName: state.displayName,
      authBusy: state.busy,
      loginHref: null,
      logoutHref: state.logoutUrl,
    }
  }
  return null
})

const hasAuthOperationError = computed(() => {
  const state = auth.state.value
  return (
    (state.status === 'authenticated' || state.status === 'unauthenticated') && state.error !== null
  )
})

const visibleJudgementState = computed<JudgementState>(() => {
  if (lastSubmission.value === 'query') return queryAnswer.state.query.state
  return queryAnswer.state.answer.state
})

const roomViewModel = computed<RoomViewModel | null>(() => {
  const run = runProblem.state.run
  const problem = runProblem.state.problem
  const room = roomDetails.value
  if (room === null || run === null || problem === null) return null

  const problemWasCleared =
    problem.status === 'cleared' ||
    run.cleared_problem_ids.includes(problem.id) ||
    queryAnswer.state.query.response?.problem_status === 'cleared' ||
    queryAnswer.state.answer.response?.problem_status === 'cleared'
  const progress = queryAnswer.state.progress

  return {
    room: {
      id: room.id,
      number: room.number,
      name: room.name,
    },
    problems: roomPageFixture.problems.map((item) => ({
      ...item,
      status:
        item.id === problem.id && problemWasCleared
          ? 'cleared'
          : queryAnswer.state.unlockedProblemIds.includes(item.id)
            ? 'available'
            : item.id === problem.id
              ? problem.status
              : item.status,
      selected: item.id === problem.id,
    })),
    selectedProblem: {
      id: problem.id,
      number: problem.number,
      type: problem.type,
      submissionType: problem.submission_type,
      title: problem.title,
      bodyMarkdown: problem.body_markdown,
      assets: problem.assets,
      hintCount: problem.hint_count,
    },
    serverElapsedMs: queryAnswer.state.elapsedMs ?? runProblem.state.elapsedMs ?? run.elapsed_ms,
    queryInput: {
      allowedControls: problem.input_schema.query.allowed_controls,
      maxOperations: problem.input_schema.query.max_operations,
      operations: bufferedOperations.value,
    },
    answerInput: {
      value: queryAnswer.state.answerInput.value,
      maxLength: problem.input_schema.answer.max_length,
    },
    queryJudgement: { state: queryAnswer.state.query.state },
    answerJudgement: { state: visibleJudgementState.value },
    clear: {
      cleared: queryAnswer.state.clear.cleared,
      clearedCount: progress?.cleared_count ?? run.cleared_problem_ids.length,
      requiredCount: progress?.required_count ?? roomPageFixture.clear.requiredCount,
    },
  }
})

onMounted(() => void auth.refresh())

watch(
  [() => route.name, () => route.params.roomId, () => auth.state.value.status],
  ([routeName, roomId, authStatus]) => {
    if (routeName !== 'room') {
      roomLoadGeneration += 1
      return
    }
    if (authStatus === 'unauthenticated') {
      roomStartRequestedFor = null
      void router.replace({ name: 'portal' })
      return
    }
    if (authStatus === 'authenticated' && typeof roomId === 'string') {
      const shouldStart = roomStartRequestedFor === roomId
      roomStartRequestedFor = null
      void loadRoom(roomId, shouldStart ? 'start' : 'restore')
    }
  },
  { immediate: true },
)

function handleLogin(): void {
  void auth.login()
}

function handleRoomSelected(roomId: string): void {
  roomStartRequestedFor = roomId
  void router.push({ name: 'room', params: { roomId } })
}

function syncBufferedOperations(): void {
  bufferedOperations.value = operationBuffer.snapshot().map((operation) => ({ ...operation }))
}

function replaceBufferedOperations(operations: readonly Operation[]): void {
  const previous = operationBuffer.snapshot()
  operationBuffer.clear(previous)
  for (const operation of operations) operationBuffer.append(operation)
  syncBufferedOperations()
}

function clearRoomInput(): void {
  queryAnswer.reset()
  syncBufferedOperations()
  lastSubmission.value = null
}

async function loadRoom(roomId: string, mode: 'restore' | 'start' = 'restore'): Promise<void> {
  const generation = ++roomLoadGeneration
  roomLoading.value = true
  roomLoadError.value = null
  roomDetails.value = null
  clearRoomInput()

  try {
    const nextRoomDetails = await client.getRoom({ room_id: roomId })
    if (generation !== roomLoadGeneration) return

    if (mode === 'start') {
      await runProblem.startOrResume(roomId)
    } else {
      try {
        await runProblem.restoreCurrentRun(roomId)
      } catch (error) {
        if (generation !== roomLoadGeneration) return
        if (!(error instanceof ApiClientError) || error.status !== 404) throw error
        await runProblem.startOrResume(roomId)
      }
    }
    if (generation !== roomLoadGeneration) return

    if (initialProblemId === null) {
      throw new Error('最初に表示する問題がfixtureにありません')
    }
    await runProblem.loadSelectedProblem(roomId, initialProblemId)
    if (generation !== roomLoadGeneration) return
    roomDetails.value = nextRoomDetails
    queryAnswer.setAnswerMaxLength(runProblem.state.problem?.input_schema.answer.max_length ?? null)
  } catch (error) {
    if (generation === roomLoadGeneration) roomLoadError.value = error
  } finally {
    if (generation === roomLoadGeneration) roomLoading.value = false
  }
}

async function selectProblem(roomId: string, problemId: string): Promise<void> {
  const generation = ++roomLoadGeneration
  roomLoading.value = true
  roomLoadError.value = null
  try {
    await runProblem.loadSelectedProblem(roomId, problemId)
    if (generation !== roomLoadGeneration) return
    queryAnswer.setAnswerMaxLength(runProblem.state.problem?.input_schema.answer.max_length ?? null)
    syncBufferedOperations()
    lastSubmission.value = null
  } catch (error) {
    if (generation === roomLoadGeneration) roomLoadError.value = error
  } finally {
    if (generation === roomLoadGeneration) roomLoading.value = false
  }
}

function removeBufferedOperation(index: number): void {
  if (submissionPending()) return
  const operations = operationBuffer.snapshot().map((operation) => ({ ...operation }))
  if (index < 0 || index >= operations.length) return
  operations.splice(index, 1)
  replaceBufferedOperations(operations)
}

async function handleRoomUiEvent(event: RoomUiEvent): Promise<void> {
  if (event.type === 'room-exited' || event.type === 'portal-returned') {
    void router.push({ name: 'portal' })
    return
  }

  const roomId = runProblem.state.roomId
  const problem = runProblem.state.problem
  const problemId = problem?.id

  if (event.type === 'condition-changed') {
    if (
      event.count <= 0 ||
      submissionPending() ||
      problem?.submission_type !== 'operation_sequence' ||
      !problem.input_schema.query.allowed_controls.includes(event.control) ||
      bufferedOperationCount() + event.count > problem.input_schema.query.max_operations
    ) {
      return
    }
    operationBuffer.append({ control: event.control, count: event.count })
    syncBufferedOperations()
    return
  }
  if (event.type === 'query-operation-removed') {
    removeBufferedOperation(event.index)
    return
  }
  if (event.type === 'query-operations-cleared') {
    if (!submissionPending()) replaceBufferedOperations([])
    return
  }
  if (event.type === 'answer-changed') {
    queryAnswer.setAnswerInput(event.value)
    return
  }
  if (roomId === null) return

  if (event.type === 'problem-selected') {
    if (submissionPending()) return
    await selectProblem(roomId, event.problemId)
    return
  }
  if (problemId === undefined) return

  if (event.type === 'query-submitted') {
    if (
      submissionPending() ||
      problem?.submission_type !== 'operation_sequence' ||
      bufferedOperations.value.length === 0
    ) {
      return
    }
    lastSubmission.value = 'query'
    try {
      await queryAnswer.submitQuery({ room_id: roomId, problem_id: problemId }, event.source)
    } catch {
      // QueryAnswerController exposes the request error through its judgement state.
    } finally {
      syncBufferedOperations()
    }
    return
  }
  if (event.type === 'answer-submitted') {
    if (submissionPending() || problem?.submission_type !== 'string') return
    lastSubmission.value = 'answer'
    try {
      await queryAnswer.submitAnswer(
        { room_id: roomId, problem_id: problemId },
        { answer: event.answer },
      )
    } catch {
      // QueryAnswerController exposes the request error through its judgement state.
    }
  }
}

function submissionPending(): boolean {
  return queryAnswer.state.query.state === 'pending' || queryAnswer.state.answer.state === 'pending'
}

function bufferedOperationCount(): number {
  return bufferedOperations.value.reduce((total, operation) => total + operation.count, 0)
}

function roomErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Roomを読み込めませんでした。'
}
</script>

<template>
  <RouterView v-slot="{ Component, route }">
    <template v-if="route.name === 'portal'">
      <p v-if="auth.state.value.status === 'loading'" role="status">認証状態を確認しています…</p>
      <p v-else-if="auth.state.value.status === 'error'" role="alert">
        認証状態を取得できませんでした。
        <button type="button" @click="auth.refresh">再試行</button>
      </p>
      <template v-else-if="portalPageProps">
        <p v-if="hasAuthOperationError" role="alert">
          認証操作に失敗しました。再度お試しください。
        </p>
        <component
          :is="Component"
          v-bind="portalPageProps"
          @login="handleLogin"
          @guest-login="auth.loginGuest"
          @logout="auth.logout"
          @start-room="handleRoomSelected"
        />
      </template>
    </template>
    <template v-else-if="route.name === 'room'">
      <p v-if="auth.state.value.status === 'loading' || roomLoading" role="status">
        Roomを読み込んでいます…
      </p>
      <p v-else-if="auth.state.value.status === 'error'" role="alert">
        認証状態を取得できませんでした。
        <button type="button" @click="auth.refresh">再試行</button>
      </p>
      <section v-else-if="roomLoadError" role="alert">
        <p>{{ roomErrorMessage(roomLoadError) }}</p>
        <button
          v-if="typeof route.params.roomId === 'string'"
          type="button"
          @click="loadRoom(route.params.roomId)"
        >
          再試行
        </button>
      </section>
      <component
        :is="Component"
        v-else-if="roomViewModel"
        :view-model="roomViewModel"
        @ui-event="handleRoomUiEvent"
      />
    </template>
    <component :is="Component" v-else />
  </RouterView>
</template>

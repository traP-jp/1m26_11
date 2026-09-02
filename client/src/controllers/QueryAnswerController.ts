import { reactive } from 'vue'

import type { JudgementState } from '@/RoomPage.types'
import type { components } from '@/generated/api'
import {
  ApiClientError,
  type ApiClient,
  type SubmitAnswerPath,
  type SubmitAnswerRequest,
  type SubmitAnswerResponse,
  type SubmitQueryPath,
  type SubmitQueryRequest,
  type SubmitQueryResponse,
} from '@/api/client'

type Progress = components['schemas']['Progress']
type RunStatus = components['schemas']['RunStatus']

export interface SubmissionState<T> {
  state: JudgementState
  response: T | null
  error: ApiClientError | null
}

export interface QueryInputState {
  source: SubmitQueryRequest['source']
  operations: SubmitQueryRequest['operations']
}

export interface AnswerInputState {
  value: string
  maxLength: number | null
}

export interface QueryAnswerControllerState {
  problemId: string | null
  queryInput: QueryInputState | null
  answerInput: AnswerInputState
  query: SubmissionState<SubmitQueryResponse>
  answer: SubmissionState<SubmitAnswerResponse>
  unlockedProblemIds: string[]
  progress: Progress | null
  runStatus: RunStatus | null
  elapsedMs: number | null
  clear: { cleared: boolean; progress: Progress | null }
}

function normalizeError(error: unknown): ApiClientError {
  if (error instanceof ApiClientError) return error

  return new ApiClientError('APIとの通信に失敗しました', {
    kind: 'network',
    cause: error,
  })
}

function emptySubmission<T>(): SubmissionState<T> {
  return { state: 'idle', response: null, error: null }
}

export class QueryAnswerController {
  readonly state = reactive<QueryAnswerControllerState>({
    problemId: null,
    queryInput: null,
    answerInput: { value: '', maxLength: null },
    query: emptySubmission<SubmitQueryResponse>(),
    answer: emptySubmission<SubmitAnswerResponse>(),
    unlockedProblemIds: [],
    progress: null,
    runStatus: null,
    elapsedMs: null,
    clear: { cleared: false, progress: null },
  })

  private queryGeneration = 0
  private answerGeneration = 0

  constructor(private readonly client: ApiClient) {}

  selectProblem(problemId: string): void {
    if (this.state.problemId === problemId) return

    this.state.problemId = problemId
    this.reset()
  }

  setQueryInput(input: SubmitQueryRequest): void {
    this.state.queryInput = {
      source: input.source,
      operations: input.operations.map((operation) => ({ ...operation })),
    }
  }

  setAnswerInput(value: string): void {
    this.state.answerInput.value = value
  }

  setAnswerMaxLength(maxLength: number | null): void {
    this.state.answerInput.maxLength = maxLength
  }

  reset(): void {
    this.queryGeneration += 1
    this.answerGeneration += 1
    this.state.queryInput = null
    this.state.answerInput = { value: '', maxLength: null }
    this.state.query = emptySubmission<SubmitQueryResponse>()
    this.state.answer = emptySubmission<SubmitAnswerResponse>()
    this.state.unlockedProblemIds = []
    this.state.progress = null
    this.state.runStatus = null
    this.state.elapsedMs = null
    this.state.clear = { cleared: false, progress: null }
  }

  async submitQuery(
    path: SubmitQueryPath,
    body: SubmitQueryRequest,
  ): Promise<SubmitQueryResponse | null> {
    this.selectProblem(path.problem_id)
    if (this.state.query.state === 'pending') return null

    this.setQueryInput(body)
    this.state.query = { state: 'pending', response: null, error: null }
    const generation = ++this.queryGeneration

    try {
      const response = await this.client.submitQuery(path, body)
      if (generation !== this.queryGeneration) return response

      this.state.query = {
        state: response.correct ? 'correct' : 'incorrect',
        response,
        error: null,
      }
      return response
    } catch (error) {
      const normalized = normalizeError(error)
      if (generation === this.queryGeneration) {
        this.state.query = { state: 'error', response: null, error: normalized }
      }
      throw normalized
    }
  }

  async submitAnswer(
    path: SubmitAnswerPath,
    body: SubmitAnswerRequest,
  ): Promise<SubmitAnswerResponse | null> {
    this.selectProblem(path.problem_id)
    if (this.state.answer.state === 'pending') return null

    this.setAnswerInput(body.answer)
    this.state.answer = { state: 'pending', response: null, error: null }
    this.state.unlockedProblemIds = []
    this.state.progress = null
    this.state.runStatus = null
    this.state.elapsedMs = null
    this.state.clear = { cleared: false, progress: null }
    const generation = ++this.answerGeneration

    try {
      const response = await this.client.submitAnswer(path, body)
      if (generation !== this.answerGeneration) return response

      this.state.answer = {
        state: response.correct ? 'correct' : 'incorrect',
        response,
        error: null,
      }
      if (response.correct) {
        this.state.unlockedProblemIds = response.unlocked_problem_ids
        this.state.progress = response.progress
        this.state.runStatus = response.run_status
        this.state.elapsedMs = response.elapsed_ms
        this.state.clear = {
          cleared: response.run_status === 'cleared',
          progress: response.progress,
        }
      } else {
        this.state.runStatus = response.run_status
      }
      return response
    } catch (error) {
      const normalized = normalizeError(error)
      if (generation === this.answerGeneration) {
        this.state.answer = { state: 'error', response: null, error: normalized }
      }
      throw normalized
    }
  }
}

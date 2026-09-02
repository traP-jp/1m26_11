import { reactive } from 'vue'

import {
  ApiClientError,
  type ApiClient,
  type GetCurrentRunResponse,
  type GetProblemResponse,
  type StartOrResumeRunResponse,
} from '@/api/client'

export type RunProblemPhase = 'idle' | 'loading' | 'ready' | 'error'
export type ProblemLoadState =
  'idle' | 'loading' | 'available' | 'cleared' | 'locked' | 'not-found' | 'error'

export interface RunProblemControllerState {
  phase: RunProblemPhase
  problemStatus: ProblemLoadState
  roomId: string | null
  run: StartOrResumeRunResponse | GetCurrentRunResponse | null
  problem: GetProblemResponse | null
  elapsedMs: number | null
  error: ApiClientError | null
}

function normalizeError(error: unknown): ApiClientError {
  if (error instanceof ApiClientError) return error

  return new ApiClientError('APIとの通信に失敗しました', {
    kind: 'network',
    cause: error,
  })
}

export class RunProblemController {
  readonly state = reactive<RunProblemControllerState>({
    phase: 'idle',
    problemStatus: 'idle',
    roomId: null,
    run: null,
    problem: null,
    elapsedMs: null,
    error: null,
  })

  constructor(private readonly client: ApiClient) {}

  async startOrResume(roomId: string): Promise<StartOrResumeRunResponse> {
    this.beginLoading(roomId)
    this.state.run = null
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.elapsedMs = null

    try {
      const run = await this.client.startOrResumeRun({ room_id: roomId })
      this.state.run = run
      this.state.elapsedMs = run.elapsed_ms
      this.state.phase = 'ready'
      return run
    } catch (error) {
      throw this.fail(error)
    }
  }

  async restoreCurrentRun(roomId: string): Promise<GetCurrentRunResponse> {
    this.beginLoading(roomId)
    this.state.run = null
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.elapsedMs = null

    try {
      const run = await this.client.getCurrentRun({ room_id: roomId })
      this.state.run = run
      this.state.elapsedMs = run.elapsed_ms
      this.state.phase = 'ready'
      return run
    } catch (error) {
      throw this.fail(error)
    }
  }

  async loadProblem(roomId: string, problemId: string): Promise<GetProblemResponse> {
    this.beginLoading(roomId)
    this.state.problem = null
    this.state.problemStatus = 'loading'

    try {
      const problem = await this.client.getProblem({ room_id: roomId, problem_id: problemId })
      this.state.problem = problem
      this.state.problemStatus = problem.status
      this.state.phase = 'ready'
      return problem
    } catch (error) {
      throw this.failProblem(error)
    }
  }

  async loadSelectedProblem(roomId: string, problemId: string): Promise<GetProblemResponse> {
    this.resetProblem()
    return this.loadProblem(roomId, problemId)
  }

  resetProblem(): void {
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.error = null
    this.state.phase = this.state.run ? 'ready' : 'idle'
  }

  private beginLoading(roomId: string): void {
    this.state.phase = 'loading'
    this.state.roomId = roomId
    this.state.error = null
  }

  private fail(error: unknown): ApiClientError {
    const normalized = normalizeError(error)
    this.state.phase = 'error'
    this.state.error = normalized
    return normalized
  }

  private failProblem(error: unknown): ApiClientError {
    const normalized = this.fail(error)
    if (normalized.code === 'PROBLEM_LOCKED' || normalized.status === 409) {
      this.state.problemStatus = 'locked'
    } else if (normalized.status === 404) {
      this.state.problemStatus = 'not-found'
    } else {
      this.state.problemStatus = 'error'
    }
    return normalized
  }
}

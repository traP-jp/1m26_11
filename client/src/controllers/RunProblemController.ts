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

export interface ProblemSelectionHandler {
  selectProblem(problemId: string): void
}

interface RequestGeneration {
  payload: number
  state: number
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

  private runRequestGeneration = 0
  private problemRequestGeneration = 0
  private stateRequestGeneration = 0

  constructor(
    private readonly client: ApiClient,
    private readonly problemSelection: ProblemSelectionHandler,
  ) {}

  async startOrResume(roomId: string): Promise<StartOrResumeRunResponse> {
    const generation = this.beginRunLoading(roomId)
    this.state.run = null
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.elapsedMs = null

    try {
      const run = await this.client.startOrResumeRun({ room_id: roomId })
      if (generation.payload === this.runRequestGeneration) {
        this.state.run = run
        this.state.elapsedMs = run.elapsed_ms
        if (generation.state === this.stateRequestGeneration) {
          this.state.phase = 'ready'
          this.state.error = null
        }
      }
      return run
    } catch (error) {
      throw this.failRun(error, generation)
    }
  }

  async restoreCurrentRun(roomId: string): Promise<GetCurrentRunResponse> {
    const generation = this.beginRunLoading(roomId)
    this.state.run = null
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.elapsedMs = null

    try {
      const run = await this.client.getCurrentRun({ room_id: roomId })
      if (generation.payload === this.runRequestGeneration) {
        this.state.run = run
        this.state.elapsedMs = run.elapsed_ms
        if (generation.state === this.stateRequestGeneration) {
          this.state.phase = 'ready'
          this.state.error = null
        }
      }
      return run
    } catch (error) {
      throw this.failRun(error, generation)
    }
  }

  async loadProblem(roomId: string, problemId: string): Promise<GetProblemResponse> {
    const generation = this.beginProblemLoading(roomId)
    this.state.problem = null
    this.state.problemStatus = 'loading'

    try {
      const path = { room_id: roomId, problem_id: problemId }
      const problem = await this.client.getProblem(path)
      const assets =
        problem.assets.length > 0 ? (await this.client.getProblemAssets(path)).items : []
      const problemWithDownloadUrls: GetProblemResponse = {
        ...problem,
        assets,
      }
      if (generation.payload === this.problemRequestGeneration) {
        this.state.problem = problemWithDownloadUrls
        this.state.problemStatus = problemWithDownloadUrls.status
        if (generation.state === this.stateRequestGeneration) {
          this.state.phase = 'ready'
          this.state.error = null
        }
      }
      return problemWithDownloadUrls
    } catch (error) {
      throw this.failProblem(error, generation)
    }
  }

  async loadSelectedProblem(roomId: string, problemId: string): Promise<GetProblemResponse> {
    this.problemSelection.selectProblem(problemId)
    this.resetProblem()
    return this.loadProblem(roomId, problemId)
  }

  resetProblem(): void {
    this.problemRequestGeneration += 1
    this.state.problem = null
    this.state.problemStatus = 'idle'
    this.state.error = null
    this.state.phase = this.state.run ? 'ready' : 'idle'
  }

  private beginRunLoading(roomId: string): RequestGeneration {
    const payload = ++this.runRequestGeneration
    this.problemRequestGeneration += 1
    const state = this.beginLoading(roomId)
    return { payload, state }
  }

  private beginProblemLoading(roomId: string): RequestGeneration {
    if (this.state.roomId !== null && this.state.roomId !== roomId) {
      this.runRequestGeneration += 1
      this.state.run = null
      this.state.elapsedMs = null
    }
    const payload = ++this.problemRequestGeneration
    const state = this.beginLoading(roomId)
    return { payload, state }
  }

  private beginLoading(roomId: string): number {
    const generation = ++this.stateRequestGeneration
    this.state.phase = 'loading'
    this.state.roomId = roomId
    this.state.error = null
    return generation
  }

  private failRun(error: unknown, generation: RequestGeneration): ApiClientError {
    const normalized = normalizeError(error)
    if (generation.payload !== this.runRequestGeneration) return normalized

    if (generation.state === this.stateRequestGeneration) {
      this.state.phase = 'error'
      this.state.error = normalized
    }
    return normalized
  }

  private failProblem(error: unknown, generation: RequestGeneration): ApiClientError {
    const normalized = normalizeError(error)
    if (generation.payload !== this.problemRequestGeneration) return normalized

    if (generation.state === this.stateRequestGeneration) {
      this.state.phase = 'error'
      this.state.error = normalized
    }

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

import { describe, expect, it, vi } from 'vitest'

import currentRun from '../../../../openapi/examples/runs/active-response.json'
import newRun from '../../../../openapi/examples/runs/start-new-response.json'
import problemResponse from '../../../../openapi/examples/problems/available-response.json'
import {
  ApiClientError,
  type ApiClient,
  type GetCurrentRunResponse,
  type GetProblemResponse,
  type StartOrResumeRunResponse,
} from '@/api/client'

import { RunProblemController } from '../RunProblemController'

const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const PROBLEM_ID = '22222222-2222-4222-8222-222222222221'
const NEXT_ROOM_ID = '11111111-1111-4111-8111-111111111112'
const NEXT_PROBLEM_ID = '22222222-2222-4222-8222-222222222222'

function createClient(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    getMe: vi.fn<ApiClient['getMe']>(),
    loginGuest: vi.fn<ApiClient['loginGuest']>(),
    logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    getRoom: vi.fn<ApiClient['getRoom']>(),
    startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
    getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
    getProblem: vi.fn<ApiClient['getProblem']>(),
    submitQuery: vi.fn<ApiClient['submitQuery']>(),
    submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
    ...overrides,
  }
}

function createProblemSelectionHandler() {
  return { selectProblem: vi.fn<(problemId: string) => void>() }
}

describe('RunProblemController', () => {
  it('starts a run and exposes the active run state', async () => {
    const startOrResumeRun = vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun }),
      createProblemSelectionHandler(),
    )

    const result = await controller.startOrResume(ROOM_ID)

    expect(startOrResumeRun).toHaveBeenCalledWith({ room_id: ROOM_ID })
    expect(result).toEqual(newRun)
    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: ROOM_ID,
      run: newRun,
      problem: null,
      error: null,
    })
  })

  it('restores the current run through the current-run endpoint', async () => {
    const getCurrentRun = vi
      .fn<ApiClient['getCurrentRun']>()
      .mockResolvedValue(currentRun as GetCurrentRunResponse)
    const controller = new RunProblemController(
      createClient({ getCurrentRun }),
      createProblemSelectionHandler(),
    )

    await controller.restoreCurrentRun(ROOM_ID)

    expect(getCurrentRun).toHaveBeenCalledWith({ room_id: ROOM_ID })
    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: ROOM_ID,
      run: currentRun,
      problem: null,
    })
  })

  it('loads a problem without discarding the active run', async () => {
    const startOrResumeRun = vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse)
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    await controller.startOrResume(ROOM_ID)
    const result = await controller.loadProblem(ROOM_ID, PROBLEM_ID)

    expect(getProblem).toHaveBeenCalledWith({ room_id: ROOM_ID, problem_id: PROBLEM_ID })
    expect(result).toEqual(problemResponse)
    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: ROOM_ID,
      run: newRun,
      problem: problemResponse,
      error: null,
    })
  })

  it('stores and rethrows an API error when loading fails', async () => {
    const apiError = new ApiClientError('問題が見つかりません', {
      kind: 'http',
      status: 404,
      code: 'PROBLEM_NOT_FOUND',
      details: {},
    })
    const getProblem = vi.fn<ApiClient['getProblem']>().mockRejectedValue(apiError)
    const controller = new RunProblemController(
      createClient({ getProblem }),
      createProblemSelectionHandler(),
    )

    await expect(controller.loadProblem(ROOM_ID, PROBLEM_ID)).rejects.toBe(apiError)

    expect(controller.state).toMatchObject({
      phase: 'error',
      roomId: ROOM_ID,
      problem: null,
      error: apiError,
    })
  })

  it('marks a locked problem separately from other loading errors', async () => {
    const apiError = new ApiClientError('問題は未解放です', {
      kind: 'http',
      status: 409,
      code: 'PROBLEM_LOCKED',
      details: {},
    })
    const getProblem = vi.fn<ApiClient['getProblem']>().mockRejectedValue(apiError)
    const controller = new RunProblemController(
      createClient({ getProblem }),
      createProblemSelectionHandler(),
    )

    await expect(controller.loadProblem(ROOM_ID, PROBLEM_ID)).rejects.toBe(apiError)

    expect(controller.state).toMatchObject({ phase: 'error', problemStatus: 'locked' })
  })

  it('marks a missing problem separately from other loading errors', async () => {
    const apiError = new ApiClientError('問題が見つかりません', {
      kind: 'http',
      status: 404,
      code: 'NOT_FOUND',
      details: {},
    })
    const getProblem = vi.fn<ApiClient['getProblem']>().mockRejectedValue(apiError)
    const controller = new RunProblemController(
      createClient({ getProblem }),
      createProblemSelectionHandler(),
    )

    await expect(controller.loadProblem(ROOM_ID, PROBLEM_ID)).rejects.toBe(apiError)

    expect(controller.state).toMatchObject({ phase: 'error', problemStatus: 'not-found' })
  })

  it('reloads a selected unlocked problem using the supplied problem id', async () => {
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const problemSelection = createProblemSelectionHandler()
    const controller = new RunProblemController(createClient({ getProblem }), problemSelection)

    await controller.loadSelectedProblem(ROOM_ID, PROBLEM_ID)

    expect(getProblem).toHaveBeenCalledWith({ room_id: ROOM_ID, problem_id: PROBLEM_ID })
    expect(problemSelection.selectProblem).toHaveBeenCalledExactlyOnceWith(PROBLEM_ID)
    expect(controller.state.problem?.id).toBe(problemResponse.id)
  })

  it('keeps the latest problem when an earlier request finishes last', async () => {
    let resolveFirstProblem!: (response: GetProblemResponse) => void
    const firstProblem = new Promise<GetProblemResponse>((resolve) => {
      resolveFirstProblem = resolve
    })
    const nextProblem = { ...problemResponse, id: NEXT_PROBLEM_ID } as GetProblemResponse
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockReturnValueOnce(firstProblem)
      .mockResolvedValueOnce(nextProblem)
    const controller = new RunProblemController(
      createClient({ getProblem }),
      createProblemSelectionHandler(),
    )

    const firstRequest = controller.loadProblem(ROOM_ID, PROBLEM_ID)
    await controller.loadProblem(ROOM_ID, NEXT_PROBLEM_ID)
    resolveFirstProblem(problemResponse as GetProblemResponse)
    await firstRequest

    expect(controller.state).toMatchObject({
      phase: 'ready',
      problemStatus: nextProblem.status,
      roomId: ROOM_ID,
      problem: nextProblem,
      error: null,
    })
  })

  it('keeps the latest run when an earlier run request finishes last', async () => {
    let resolveStart!: (response: StartOrResumeRunResponse) => void
    const pendingStart = new Promise<StartOrResumeRunResponse>((resolve) => {
      resolveStart = resolve
    })
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>().mockReturnValue(pendingStart)
    const getCurrentRun = vi
      .fn<ApiClient['getCurrentRun']>()
      .mockResolvedValue(currentRun as GetCurrentRunResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getCurrentRun }),
      createProblemSelectionHandler(),
    )

    const firstRequest = controller.startOrResume(ROOM_ID)
    await controller.restoreCurrentRun(NEXT_ROOM_ID)
    resolveStart(newRun as StartOrResumeRunResponse)
    await firstRequest

    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: NEXT_ROOM_ID,
      run: currentRun,
      elapsedMs: currentRun.elapsed_ms,
      error: null,
    })
  })

  it('keeps a pending run request active when a problem request starts', async () => {
    let resolveStart!: (response: StartOrResumeRunResponse) => void
    const pendingStart = new Promise<StartOrResumeRunResponse>((resolve) => {
      resolveStart = resolve
    })
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>().mockReturnValue(pendingStart)
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    const runRequest = controller.startOrResume(ROOM_ID)
    await controller.loadProblem(ROOM_ID, PROBLEM_ID)
    resolveStart(newRun as StartOrResumeRunResponse)
    await runRequest

    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: ROOM_ID,
      run: newRun,
      problem: problemResponse,
      error: null,
    })
  })

  it('does not restore a stale problem after starting another run', async () => {
    let resolveProblem!: (response: GetProblemResponse) => void
    const pendingProblem = new Promise<GetProblemResponse>((resolve) => {
      resolveProblem = resolve
    })
    const getProblem = vi.fn<ApiClient['getProblem']>().mockReturnValue(pendingProblem)
    const startOrResumeRun = vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse)
    const controller = new RunProblemController(
      createClient({ getProblem, startOrResumeRun }),
      createProblemSelectionHandler(),
    )

    const problemRequest = controller.loadProblem(ROOM_ID, PROBLEM_ID)
    await controller.startOrResume(NEXT_ROOM_ID)
    resolveProblem(problemResponse as GetProblemResponse)
    await problemRequest

    expect(controller.state).toMatchObject({
      phase: 'ready',
      problemStatus: 'idle',
      roomId: NEXT_ROOM_ID,
      run: newRun,
      problem: null,
      error: null,
    })
  })

  it('stays loading until the latest problem request finishes', async () => {
    let resolveStart!: (response: StartOrResumeRunResponse) => void
    let resolveProblem!: (response: GetProblemResponse) => void
    const pendingStart = new Promise<StartOrResumeRunResponse>((resolve) => {
      resolveStart = resolve
    })
    const pendingProblem = new Promise<GetProblemResponse>((resolve) => {
      resolveProblem = resolve
    })
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>().mockReturnValue(pendingStart)
    const getProblem = vi.fn<ApiClient['getProblem']>().mockReturnValue(pendingProblem)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    const runRequest = controller.startOrResume(ROOM_ID)
    const problemRequest = controller.loadProblem(ROOM_ID, PROBLEM_ID)
    resolveStart(newRun as StartOrResumeRunResponse)
    await runRequest

    expect(controller.state).toMatchObject({
      phase: 'loading',
      problemStatus: 'loading',
      run: newRun,
      problem: null,
      error: null,
    })

    resolveProblem(problemResponse as GetProblemResponse)
    await problemRequest
    expect(controller.state.phase).toBe('ready')
  })

  it('keeps the latest problem error when an earlier run succeeds later', async () => {
    let resolveStart!: (response: StartOrResumeRunResponse) => void
    const pendingStart = new Promise<StartOrResumeRunResponse>((resolve) => {
      resolveStart = resolve
    })
    const apiError = new ApiClientError('問題が見つかりません', {
      kind: 'http',
      status: 404,
      code: 'PROBLEM_NOT_FOUND',
      details: {},
    })
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>().mockReturnValue(pendingStart)
    const getProblem = vi.fn<ApiClient['getProblem']>().mockRejectedValue(apiError)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    const runRequest = controller.startOrResume(ROOM_ID)
    await expect(controller.loadProblem(ROOM_ID, PROBLEM_ID)).rejects.toBe(apiError)
    resolveStart(newRun as StartOrResumeRunResponse)
    await runRequest

    expect(controller.state).toMatchObject({
      phase: 'error',
      problemStatus: 'not-found',
      run: newRun,
      problem: null,
      error: apiError,
    })
  })

  it('does not attach a pending run to a problem loaded from another room', async () => {
    let resolveStart!: (response: StartOrResumeRunResponse) => void
    const pendingStart = new Promise<StartOrResumeRunResponse>((resolve) => {
      resolveStart = resolve
    })
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>().mockReturnValue(pendingStart)
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    const runRequest = controller.startOrResume(ROOM_ID)
    await controller.loadProblem(NEXT_ROOM_ID, PROBLEM_ID)
    resolveStart(newRun as StartOrResumeRunResponse)
    await runRequest

    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: NEXT_ROOM_ID,
      run: null,
      problem: problemResponse,
      error: null,
    })
  })

  it('clears the current run when loading a problem from another room', async () => {
    const startOrResumeRun = vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse)
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const controller = new RunProblemController(
      createClient({ startOrResumeRun, getProblem }),
      createProblemSelectionHandler(),
    )

    await controller.startOrResume(ROOM_ID)
    await controller.loadProblem(NEXT_ROOM_ID, PROBLEM_ID)

    expect(controller.state).toMatchObject({
      phase: 'ready',
      roomId: NEXT_ROOM_ID,
      run: null,
      elapsedMs: null,
      problem: problemResponse,
      error: null,
    })
  })
})

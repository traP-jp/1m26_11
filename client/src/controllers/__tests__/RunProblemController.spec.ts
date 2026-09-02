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

function createClient(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    getMe: vi.fn<ApiClient['getMe']>(),
    loginGuest: vi.fn<ApiClient['loginGuest']>(),
    logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
    getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
    getProblem: vi.fn<ApiClient['getProblem']>(),
    submitQuery: vi.fn<ApiClient['submitQuery']>(),
    submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
    ...overrides,
  }
}

describe('RunProblemController', () => {
  it('starts a run and exposes the active run state', async () => {
    const startOrResumeRun = vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse)
    const controller = new RunProblemController(createClient({ startOrResumeRun }))

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
    const controller = new RunProblemController(createClient({ getCurrentRun }))

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
    const controller = new RunProblemController(createClient({ startOrResumeRun, getProblem }))

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
    const controller = new RunProblemController(createClient({ getProblem }))

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
    const controller = new RunProblemController(createClient({ getProblem }))

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
    const controller = new RunProblemController(createClient({ getProblem }))

    await expect(controller.loadProblem(ROOM_ID, PROBLEM_ID)).rejects.toBe(apiError)

    expect(controller.state).toMatchObject({ phase: 'error', problemStatus: 'not-found' })
  })

  it('reloads a selected unlocked problem using the supplied problem id', async () => {
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const controller = new RunProblemController(createClient({ getProblem }))

    await controller.loadSelectedProblem(ROOM_ID, PROBLEM_ID)

    expect(getProblem).toHaveBeenCalledWith({ room_id: ROOM_ID, problem_id: PROBLEM_ID })
    expect(controller.state.problem?.id).toBe(problemResponse.id)
  })
})

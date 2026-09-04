import { describe, expect, it, vi } from 'vitest'

import answerCorrect from '../../../../openapi/examples/answers/response-correct-unlock.json'
import answerCorrectAndClearsRun from '../../../../openapi/examples/answers/response-correct-cleared.json'
import answerIncorrect from '../../../../openapi/examples/answers/response-incorrect.json'
import answerRequest from '../../../../openapi/examples/answers/request.json'
import queryCorrect from '../../../../openapi/examples/queries/response-correct.json'
import queryIncorrect from '../../../../openapi/examples/queries/response-incorrect.json'
import queryRequest from '../../../../openapi/examples/queries/request-serial.json'
import {
  ApiClientError,
  type ApiClient,
  type SubmitAnswerResponse,
  type SubmitQueryResponse,
} from '@/api/client'
import { createOperationBuffer, type OperationBuffer } from '@/input/operationBuffer'

import { QueryAnswerController } from '../QueryAnswerController'

const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const PROBLEM_ID = '22222222-2222-4222-8222-222222222221'
const NEXT_PROBLEM_ID = '22222222-2222-4222-8222-222222222222'
const PROBLEM_PATH = { room_id: ROOM_ID, problem_id: PROBLEM_ID }

function appendQueryOperations(buffer: OperationBuffer): void {
  for (const operation of queryRequest.operations) buffer.append(operation)
}

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
    createProblem: vi.fn<ApiClient['createProblem']>(),
    uploadProblemAsset: vi.fn<ApiClient['uploadProblemAsset']>(),
    ...overrides,
  }
}

describe('QueryAnswerController', () => {
  it('submits one query and maps a correct response to correct judgement', async () => {
    const submitQuery = vi
      .fn<ApiClient['submitQuery']>()
      .mockResolvedValue(queryCorrect as SubmitQueryResponse)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient({ submitQuery }), buffer)
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    const result = await controller.submitQuery(PROBLEM_PATH, queryRequest.source)

    expect(submitQuery).toHaveBeenCalledExactlyOnceWith(PROBLEM_PATH, queryRequest)
    expect(result).toEqual(queryCorrect)
    expect(controller.state.query).toMatchObject({
      state: 'correct',
      response: queryCorrect,
      error: null,
    })
    expect(controller.state.queryInput).toEqual(queryRequest)
    expect(buffer.snapshot()).toEqual([])
  })

  it('maps an incorrect answer response to incorrect judgement', async () => {
    const submitAnswer = vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerIncorrect as SubmitAnswerResponse)
    const controller = new QueryAnswerController(
      createClient({ submitAnswer }),
      createOperationBuffer(),
    )

    const result = await controller.submitAnswer(PROBLEM_PATH, answerRequest)

    expect(submitAnswer).toHaveBeenCalledWith(PROBLEM_PATH, answerRequest)
    expect(result).toEqual(answerIncorrect)
    expect(controller.state.answer).toMatchObject({
      state: 'incorrect',
      response: answerIncorrect,
      error: null,
    })
    expect(controller.state.answerInput.value).toBe(answerRequest.answer)
  })

  it('exposes unlock, progress, elapsed time, and clear state from a correct answer', async () => {
    const submitAnswer = vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerCorrect as SubmitAnswerResponse)
    const controller = new QueryAnswerController(
      createClient({ submitAnswer }),
      createOperationBuffer(),
    )

    await controller.submitAnswer(PROBLEM_PATH, answerRequest)

    expect(controller.state).toMatchObject({
      unlockedProblemIds: answerCorrect.unlocked_problem_ids,
      progress: answerCorrect.progress,
      runStatus: answerCorrect.run_status,
      elapsedMs: answerCorrect.elapsed_ms,
      clear: { cleared: false },
    })
  })

  it('marks the run clear only when the answer response says the run is cleared', async () => {
    const submitAnswer = vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerCorrectAndClearsRun as SubmitAnswerResponse)
    const controller = new QueryAnswerController(
      createClient({ submitAnswer }),
      createOperationBuffer(),
    )

    await controller.submitAnswer(PROBLEM_PATH, answerRequest)

    expect(controller.state).toMatchObject({
      runStatus: 'cleared',
      elapsedMs: answerCorrectAndClearsRun.elapsed_ms,
      clear: { cleared: true, progress: answerCorrectAndClearsRun.progress },
    })
  })

  it('prevents a second answer request while the first answer is pending', async () => {
    let resolveAnswer!: (response: SubmitAnswerResponse) => void
    const pendingAnswer = new Promise<SubmitAnswerResponse>((resolve) => {
      resolveAnswer = resolve
    })
    const submitAnswer = vi.fn<ApiClient['submitAnswer']>().mockReturnValue(pendingAnswer)
    const controller = new QueryAnswerController(
      createClient({ submitAnswer }),
      createOperationBuffer(),
    )

    const firstRequest = controller.submitAnswer(PROBLEM_PATH, answerRequest)
    const duplicateResult = await controller.submitAnswer(PROBLEM_PATH, answerRequest)

    expect(duplicateResult).toBeNull()
    expect(submitAnswer).toHaveBeenCalledOnce()
    expect(controller.state.answer.state).toBe('pending')

    resolveAnswer(answerCorrect as SubmitAnswerResponse)
    await firstRequest
  })

  it('prevents a second query request while the first query is pending', async () => {
    let resolveQuery!: (response: SubmitQueryResponse) => void
    const pendingQuery = new Promise<SubmitQueryResponse>((resolve) => {
      resolveQuery = resolve
    })
    const submitQuery = vi.fn<ApiClient['submitQuery']>().mockReturnValue(pendingQuery)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient({ submitQuery }), buffer)
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    const firstRequest = controller.submitQuery(PROBLEM_PATH, queryRequest.source)
    const duplicateResult = await controller.submitQuery(PROBLEM_PATH, queryRequest.source)

    expect(duplicateResult).toBeNull()
    expect(submitQuery).toHaveBeenCalledOnce()
    expect(controller.state.query.state).toBe('pending')

    resolveQuery(queryCorrect as SubmitQueryResponse)
    await firstRequest
  })

  it('keeps operations appended while a query request is pending', async () => {
    let resolveQuery!: (response: SubmitQueryResponse) => void
    const pendingQuery = new Promise<SubmitQueryResponse>((resolve) => {
      resolveQuery = resolve
    })
    const submitQuery = vi.fn<ApiClient['submitQuery']>().mockReturnValue(pendingQuery)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient({ submitQuery }), buffer)
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    const request = controller.submitQuery(PROBLEM_PATH, queryRequest.source)
    buffer.append({ control: 'left', count: 1 })
    resolveQuery(queryCorrect as SubmitQueryResponse)
    await request

    expect(buffer.snapshot()).toEqual([{ control: 'left', count: 1 }])
  })

  it('does not let a previous problem response overwrite the newly selected problem', async () => {
    let resolveFirstQuery!: (response: SubmitQueryResponse) => void
    const firstQuery = new Promise<SubmitQueryResponse>((resolve) => {
      resolveFirstQuery = resolve
    })
    const submitQuery = vi
      .fn<ApiClient['submitQuery']>()
      .mockReturnValueOnce(firstQuery)
      .mockResolvedValueOnce(queryCorrect as SubmitQueryResponse)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient({ submitQuery }), buffer)
    const nextProblemPath = { room_id: ROOM_ID, problem_id: NEXT_PROBLEM_ID }
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    const firstRequest = controller.submitQuery(PROBLEM_PATH, queryRequest.source)
    controller.selectProblem(NEXT_PROBLEM_ID)
    appendQueryOperations(buffer)
    await controller.submitQuery(nextProblemPath, queryRequest.source)
    resolveFirstQuery(queryIncorrect as SubmitQueryResponse)
    await firstRequest

    expect(controller.state).toMatchObject({
      problemId: NEXT_PROBLEM_ID,
      query: { state: 'correct', response: queryCorrect, error: null },
    })
  })

  it('stores and rethrows an API error', async () => {
    const apiError = new ApiClientError('入力が正しくありません', {
      kind: 'http',
      status: 422,
      code: 'VALIDATION_ERROR',
      details: {},
    })
    const submitQuery = vi.fn<ApiClient['submitQuery']>().mockRejectedValue(apiError)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient({ submitQuery }), buffer)
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    await expect(controller.submitQuery(PROBLEM_PATH, queryRequest.source)).rejects.toBe(apiError)

    expect(controller.state.query).toMatchObject({
      state: 'error',
      response: null,
      error: apiError,
    })
    expect(buffer.snapshot()).toEqual(queryRequest.operations)
  })

  it('distinguishes a network error from an incorrect response', async () => {
    const apiError = new ApiClientError('APIとの通信に失敗しました', {
      kind: 'network',
      cause: new Error('connection lost'),
    })
    const submitAnswer = vi.fn<ApiClient['submitAnswer']>().mockRejectedValue(apiError)
    const controller = new QueryAnswerController(
      createClient({ submitAnswer }),
      createOperationBuffer(),
    )

    await expect(controller.submitAnswer(PROBLEM_PATH, answerRequest)).rejects.toBe(apiError)

    expect(controller.state.answer).toMatchObject({
      state: 'error',
      response: null,
      error: apiError,
    })
  })

  it('resets unsent query operations and answer input when selecting another problem', () => {
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(createClient(), buffer)

    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)
    controller.setQueryInput(queryRequest)
    controller.setAnswerInput(answerRequest.answer)
    controller.setAnswerMaxLength(50)
    controller.selectProblem(NEXT_PROBLEM_ID)

    expect(controller.state).toMatchObject({
      problemId: NEXT_PROBLEM_ID,
      queryInput: null,
      answerInput: { value: '', maxLength: null },
    })
    expect(buffer.snapshot()).toEqual([])
  })

  it('resets both judgements when a different problem is selected', async () => {
    const submitQuery = vi
      .fn<ApiClient['submitQuery']>()
      .mockResolvedValue(queryCorrect as SubmitQueryResponse)
    const submitAnswer = vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerIncorrect as SubmitAnswerResponse)
    const buffer = createOperationBuffer()
    const controller = new QueryAnswerController(
      createClient({ submitQuery, submitAnswer }),
      buffer,
    )
    controller.selectProblem(PROBLEM_ID)
    appendQueryOperations(buffer)

    await controller.submitQuery(PROBLEM_PATH, queryRequest.source)
    await controller.submitAnswer(PROBLEM_PATH, answerRequest)
    controller.selectProblem(NEXT_PROBLEM_ID)

    expect(controller.state).toMatchObject({
      problemId: NEXT_PROBLEM_ID,
      query: { state: 'idle', response: null, error: null },
      answer: { state: 'idle', response: null, error: null },
    })
  })
})

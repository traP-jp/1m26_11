import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { setupServer } from 'msw/node'

import answerRequest from '../../../../openapi/examples/answers/request.json'
import answerCorrectAndClearsRun from '../../../../openapi/examples/answers/response-correct-cleared.json'
import problemResponse from '../../../../openapi/examples/problems/available-response.json'
import queryRequest from '../../../../openapi/examples/queries/request-serial.json'
import { createApiClient } from '@/api/client'
import { createOperationBuffer } from '@/input/operationBuffer'
import { createMockApi } from '@/mocks/handlers'

import { QueryAnswerController } from '../QueryAnswerController'
import { RunProblemController } from '../RunProblemController'

const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const PROBLEM_ID = '22222222-2222-4222-8222-222222222221'
const NEXT_PROBLEM_ID = '22222222-2222-4222-8222-222222222222'
const PROBLEM_PATH = { room_id: ROOM_ID, problem_id: PROBLEM_ID }

const server = setupServer()

beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' })
})

afterEach(() => {
  server.resetHandlers()
})

afterAll(() => {
  server.close()
})

describe('controllers with the OpenAPI-backed mock API', () => {
  it('starts, restores, loads, and clears a run through the real ApiClient boundary', async () => {
    const mock = createMockApi({ scenarioId: 'start_new_run' })
    server.use(...mock.handlers)
    const client = createApiClient({ baseUrl: window.location.origin })
    const operationBuffer = createOperationBuffer()
    const queryAnswer = new QueryAnswerController(client, operationBuffer)
    const runProblem = new RunProblemController(client, queryAnswer)

    await runProblem.startOrResume(ROOM_ID)
    await runProblem.restoreCurrentRun(ROOM_ID)
    await runProblem.loadSelectedProblem(ROOM_ID, PROBLEM_ID)

    expect(runProblem.state).toMatchObject({
      phase: 'ready',
      problemStatus: 'available',
      run: { status: 'active', elapsed_ms: 65000 },
      problem: problemResponse,
      elapsedMs: 65000,
    })

    mock.state.patch({
      problem_status: 'available',
      next_problem_status: 'locked',
      query_judgement: 'correct',
    })
    for (const operation of queryRequest.operations) operationBuffer.append(operation)
    await queryAnswer.submitQuery(PROBLEM_PATH, queryRequest.source)

    expect(queryAnswer.state.query).toMatchObject({ state: 'correct' })

    mock.state.patch({
      problem_status: 'available',
      answer_judgement: 'correct',
      last_required_problem: true,
    })
    await queryAnswer.submitAnswer(PROBLEM_PATH, answerRequest)

    expect(queryAnswer.state).toMatchObject({
      runStatus: 'cleared',
      elapsedMs: 119820,
      clear: { cleared: true },
      progress: answerCorrectAndClearsRun.progress,
    })
  })

  it('resets query state and buffered operations before loading another problem', async () => {
    const mock = createMockApi({ scenarioId: 'start_new_run' })
    server.use(...mock.handlers)
    const client = createApiClient({ baseUrl: window.location.origin })
    const operationBuffer = createOperationBuffer()
    const queryAnswer = new QueryAnswerController(client, operationBuffer)
    const runProblem = new RunProblemController(client, queryAnswer)

    await runProblem.loadSelectedProblem(ROOM_ID, PROBLEM_ID)
    for (const operation of queryRequest.operations) operationBuffer.append(operation)
    queryAnswer.setQueryInput(queryRequest)
    queryAnswer.setAnswerInput(answerRequest.answer)

    await runProblem.loadSelectedProblem(ROOM_ID, NEXT_PROBLEM_ID)

    expect(queryAnswer.state).toMatchObject({
      problemId: NEXT_PROBLEM_ID,
      queryInput: null,
      answerInput: { value: '', maxLength: null },
    })
    expect(operationBuffer.snapshot()).toEqual([])
  })
})

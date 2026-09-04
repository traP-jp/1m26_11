import { HttpResponse, http } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'

import answerRequest from '../../../../openapi/examples/answers/request.json'
import answerIncorrect from '../../../../openapi/examples/answers/response-incorrect.json'
import unauthorized from '../../../../openapi/examples/auth/error-unauthorized.json'
import guestRequest from '../../../../openapi/examples/auth/guest-request.json'
import guestResponse from '../../../../openapi/examples/auth/guest-response.json'
import meAuthenticated from '../../../../openapi/examples/auth/me-demo-authenticated.json'
import problemLocked from '../../../../openapi/examples/problems/error-problem-locked.json'
import problemResponse from '../../../../openapi/examples/problems/available-response.json'
import queryRequest from '../../../../openapi/examples/queries/request-serial.json'
import queryIncorrect from '../../../../openapi/examples/queries/response-incorrect.json'
import currentRun from '../../../../openapi/examples/runs/active-response.json'
import runNotFound from '../../../../openapi/examples/runs/error-run-not-found.json'
import newRun from '../../../../openapi/examples/runs/start-new-response.json'
import { ApiClientError, createApiClient, type CreateProblemRequest } from '@/api/client'
import assetCreated from '../../../../openapi/examples/assets/response-created.json'
import createProblemRequest from '../../../../openapi/examples/problems/create-string-request.json'
import createProblemResponse from '../../../../openapi/examples/problems/create-response.json'

const API_ORIGIN = 'https://api.example.test'
const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const PROBLEM_ID = '22222222-2222-4222-8222-222222222221'
const IDEMPOTENCY_KEY = '44444444-4444-4444-8444-444444444444'

const runPath = { room_id: ROOM_ID }
const problemPath = { room_id: ROOM_ID, problem_id: PROBLEM_ID }
const createProblemPath = { room_id: ROOM_ID }
const idempotencyHeaders = { 'Idempotency-Key': IDEMPOTENCY_KEY }
const typedCreateProblemRequest = createProblemRequest as CreateProblemRequest
const server = setupServer()
let client: ReturnType<typeof createApiClient>

interface ObservedRequest {
  url: string
  method: string
  credentials: RequestCredentials
  contentType: string | null
  idempotencyKey: string | null
  body: string
}

function captureResponse(response: Response): () => ObservedRequest {
  let observed: ObservedRequest | undefined

  server.use(
    http.all('*', async ({ request }) => {
      observed = {
        url: request.url,
        method: request.method,
        credentials: request.credentials,
        contentType: request.headers.get('content-type'),
        idempotencyKey: request.headers.get('idempotency-key'),
        body: await request.text(),
      }
      return response
    }),
  )

  return () => {
    if (!observed) throw new Error('ApiClient did not send a request')
    return observed
  }
}

async function catchApiClientError(invoke: () => Promise<unknown>): Promise<ApiClientError> {
  let caught: unknown

  try {
    await invoke()
  } catch (error) {
    caught = error
  }

  if (caught instanceof ApiClientError) return caught
  if (caught !== undefined) throw caught
  throw new Error('ApiClient request unexpectedly succeeded')
}

beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' })
  client = createApiClient({ baseUrl: `${API_ORIGIN}/` })
})

afterEach(() => {
  server.resetHandlers()
})

afterAll(() => {
  server.close()
})

describe('ApiClient', () => {
  const successCases = [
    {
      name: 'getMe',
      method: 'GET',
      path: '/api/me',
      invoke: () => client.getMe(),
      response: HttpResponse.json(meAuthenticated),
      expectedResult: meAuthenticated,
    },
    {
      name: 'loginGuest',
      method: 'POST',
      path: '/api/auth/guest',
      invoke: () => client.loginGuest(guestRequest),
      response: HttpResponse.json(guestResponse),
      expectedResult: guestResponse,
      expectedBody: guestRequest,
    },
    {
      name: 'logoutDemo',
      method: 'POST',
      path: '/api/auth/logout',
      invoke: () => client.logoutDemo(),
      response: new HttpResponse(null, { status: 204 }),
      expectedResult: undefined,
    },
    {
      name: 'startOrResumeRun',
      method: 'POST',
      path: `/api/rooms/${ROOM_ID}/runs`,
      invoke: () => client.startOrResumeRun(runPath),
      response: HttpResponse.json(newRun),
      expectedResult: newRun,
    },
    {
      name: 'getCurrentRun',
      method: 'GET',
      path: `/api/rooms/${ROOM_ID}/runs/current`,
      invoke: () => client.getCurrentRun(runPath),
      response: HttpResponse.json(currentRun),
      expectedResult: currentRun,
    },
    {
      name: 'getProblem',
      method: 'GET',
      path: `/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}`,
      invoke: () => client.getProblem(problemPath),
      response: HttpResponse.json(problemResponse),
      expectedResult: problemResponse,
    },
    {
      name: 'submitQuery',
      method: 'POST',
      path: `/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/queries`,
      invoke: () => client.submitQuery(problemPath, queryRequest),
      response: HttpResponse.json(queryIncorrect),
      expectedResult: queryIncorrect,
      expectedBody: queryRequest,
    },
    {
      name: 'submitAnswer',
      method: 'POST',
      path: `/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/answers`,
      invoke: () => client.submitAnswer(problemPath, answerRequest),
      response: HttpResponse.json(answerIncorrect),
      expectedResult: answerIncorrect,
      expectedBody: answerRequest,
    },
  ]

  it.each(successCases)(
    '$name sends the contract-shaped request and returns the success response',
    async ({ method, path, invoke, response, expectedResult, expectedBody }) => {
      const getObserved = captureResponse(response)

      const result = await invoke()

      expect(result).toEqual(expectedResult)

      const observed = getObserved()
      expect(observed).toMatchObject({
        url: `${API_ORIGIN}${path}`,
        method,
        credentials: 'include',
      })
      expect(observed.contentType).toBe(expectedBody === undefined ? null : 'application/json')
      expect(observed.body).toBe(expectedBody === undefined ? '' : JSON.stringify(expectedBody))
    },
  )

  it('creates a problem with the contract body and idempotency key', async () => {
    const getObserved = captureResponse(
      HttpResponse.json(createProblemResponse, {
        status: 201,
      }),
    )

    const result = await client.createProblem(
      createProblemPath,
      idempotencyHeaders,
      typedCreateProblemRequest,
    )

    expect(result).toEqual(createProblemResponse)
    expect(getObserved()).toMatchObject({
      url: `${API_ORIGIN}/api/rooms/${ROOM_ID}/problems`,
      method: 'POST',
      credentials: 'include',
      contentType: 'application/json',
      idempotencyKey: IDEMPOTENCY_KEY,
      body: JSON.stringify(typedCreateProblemRequest),
    })
  })

  it('uploads a problem image as multipart data without setting content-type manually', async () => {
    const getObserved = captureResponse(
      HttpResponse.json(assetCreated, {
        status: 201,
      }),
    )
    const file = new File(['api-client-image'], 'question.png', {
      type: 'image/png',
    })

    const result = await client.uploadProblemAsset(
      {
        room_id: ROOM_ID,
        problem_id: PROBLEM_ID,
      },
      idempotencyHeaders,
      {
        file,
        alt: '問題画像',
      },
    )

    expect(result).toEqual(assetCreated)

    const observed = getObserved()
    expect(observed).toMatchObject({
      url: `${API_ORIGIN}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/assets`,
      method: 'POST',
      credentials: 'include',
      idempotencyKey: IDEMPOTENCY_KEY,
    })
    expect(observed.contentType).toMatch(/^multipart\/form-data; boundary=/)
    expect(observed.body).toContain('name="file"')
    expect(observed.body).toContain('name="alt"')
    expect(observed.body).toContain('問題画像')
  })

  it('uses the current origin when baseUrl is omitted', async () => {
    const defaultClient = createApiClient()
    const getObserved = captureResponse(HttpResponse.json(meAuthenticated))

    await defaultClient.getMe()

    expect(getObserved().url).toBe(`${window.location.origin}/api/me`)
  })

  const httpErrorCases = [
    {
      name: 'unauthorized',
      status: 401,
      body: unauthorized,
      invoke: () => client.startOrResumeRun(runPath),
    },
    {
      name: 'missing current run',
      status: 404,
      body: runNotFound,
      invoke: () => client.getCurrentRun(runPath),
    },
    {
      name: 'locked problem',
      status: 409,
      body: problemLocked,
      invoke: () => client.getProblem(problemPath),
    },
  ]

  it.each(httpErrorCases)(
    'normalizes the $name response into an ApiClientError',
    async ({ status, body, invoke }) => {
      captureResponse(HttpResponse.json(body, { status }))

      const error = await catchApiClientError(invoke)

      expect(error).toMatchObject({
        kind: 'http',
        status,
        code: body.error.code,
        message: body.error.message,
        details: body.error.details,
      })
      expect(error.failure).toEqual({
        kind: 'http',
        status,
        code: body.error.code,
        details: body.error.details,
      })
    },
  )

  it('normalizes a network failure without inventing HTTP information', async () => {
    captureResponse(HttpResponse.error())

    const error = await catchApiClientError(() => client.getMe())

    expect(error).toMatchObject({ kind: 'network' })
    expect(error.status).toBeUndefined()
    expect(error.code).toBeUndefined()
    expect(error.cause).toBeDefined()
    expect(error.failure).toMatchObject({ kind: 'network' })
  })

  it('normalizes invalid JSON while retaining the response status', async () => {
    captureResponse(
      HttpResponse.text('{', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    )

    const error = await catchApiClientError(() => client.getMe())

    expect(error).toMatchObject({
      kind: 'invalid-response',
      status: 200,
    })
    expect(error.code).toBeUndefined()
    expect(error.cause).toBeDefined()
  })

  it('normalizes invalid JSON in an HTTP error while retaining its status', async () => {
    captureResponse(
      HttpResponse.text('{', {
        status: 500,
        headers: { 'content-type': 'application/json' },
      }),
    )

    const error = await catchApiClientError(() => client.getMe())

    expect(error).toMatchObject({
      kind: 'invalid-response',
      status: 500,
    })
    expect(error.code).toBeUndefined()
    expect(error.cause).toBeDefined()
  })

  it('normalizes a JSON error body that does not match ErrorResponse', async () => {
    captureResponse(HttpResponse.json({ message: 'broken' }, { status: 500 }))

    const error = await catchApiClientError(() => client.getMe())

    expect(error).toMatchObject({
      kind: 'invalid-response',
      status: 500,
    })
    expect(error.code).toBeUndefined()
  })

  const unexpectedSuccessCases = [
    {
      name: 'JSON endpoint',
      status: 201,
      response: HttpResponse.json(meAuthenticated, { status: 201 }),
      invoke: () => client.getMe(),
    },
    {
      name: 'No Content endpoint',
      status: 200,
      response: HttpResponse.json({}),
      invoke: () => client.logoutDemo(),
    },
  ]

  it.each(unexpectedSuccessCases)(
    'rejects an unexpected success status from the $name',
    async ({ status, response, invoke }) => {
      captureResponse(response)

      const error = await catchApiClientError(invoke)

      expect(error).toMatchObject({
        kind: 'invalid-response',
        status,
      })
    },
  )
})

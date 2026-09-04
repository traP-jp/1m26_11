import { afterEach, describe, expect, it } from 'vitest'
import { setupServer } from 'msw/node'

import imageNotFound from '../../../../openapi/examples/assets/error-image-not-found.json'
import problemAssets from '../../../../openapi/examples/assets/response-list.json'
import answerCleared from '../../../../openapi/examples/answers/response-correct-cleared.json'
import displayNameRequired from '../../../../openapi/examples/auth/error-display-name-required.json'
import displayNameTooLong from '../../../../openapi/examples/auth/error-display-name-too-long.json'
import guestLogin from '../../../../openapi/examples/auth/guest-request.json'
import meAuthenticated from '../../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../../openapi/examples/auth/me-demo-unauthenticated.json'
import unauthorized from '../../../../openapi/examples/auth/error-unauthorized.json'
import problemLocked from '../../../../openapi/examples/problems/error-problem-locked.json'
import queryCorrect from '../../../../openapi/examples/queries/response-correct.json'
import currentRun from '../../../../openapi/examples/runs/active-response.json'
import runNotFound from '../../../../openapi/examples/runs/error-run-not-found.json'
import newRun from '../../../../openapi/examples/runs/start-new-response.json'
import openApiSource from '../../../../openapi/openapi-v1.yaml?raw'
import scenarioSource from '../../../../openapi/scenarios/p0-cases.yaml?raw'
import { createMockContract, type MockContract } from '../contract'
import { createMockApi, type MockApi } from '../handlers'

const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const PROBLEM_ID = '22222222-2222-4222-8222-222222222221'
const BASE_URL = window.location.origin
const fixtureModules = import.meta.glob('../../../../openapi/examples/**/*.json', {
  eager: true,
  import: 'default',
})

let server: ReturnType<typeof setupServer> | undefined

function startMock(scenarioId?: string, contract?: MockContract): MockApi {
  const mock = createMockApi({ scenarioId, contract })
  const nextServer = setupServer(...mock.handlers)
  nextServer.listen({ onUnhandledRequest: 'error' })
  server = nextServer
  return mock
}

afterEach(() => {
  server?.close()
  server = undefined
})

describe('OpenAPI-backed MSW handlers', () => {
  it('serves the OpenAPI document and authenticated demo user by default', async () => {
    startMock()

    const contractResponse = await fetch(`${BASE_URL}/openapi.yaml`)
    expect(contractResponse.status).toBe(200)
    expect(contractResponse.headers.get('content-type')).toContain('application/yaml')
    expect(await contractResponse.text()).toContain('openapi: 3.1.0')

    const meResponse = await fetch(`${BASE_URL}/api/me`)
    expect(await meResponse.json()).toEqual(meAuthenticated)
  })

  it('starts a new run, then resumes it as the current run', async () => {
    startMock()

    const startResponse = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/runs`, {
      method: 'POST',
    })
    expect(await startResponse.json()).toEqual(newRun)

    const currentResponse = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/runs/current`)
    expect(await currentResponse.json()).toEqual(currentRun)
  })

  it('follows demo login and logout state transitions', async () => {
    startMock('demo_login_and_logout')

    expect(await (await fetch(`${BASE_URL}/api/me`)).json()).toEqual(meUnauthenticated)

    const loginResponse = await fetch(`${BASE_URL}/api/auth/guest`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(guestLogin),
    })
    expect(loginResponse.status).toBe(200)
    expect(await (await fetch(`${BASE_URL}/api/me`)).json()).toEqual(meAuthenticated)

    const logoutResponse = await fetch(`${BASE_URL}/api/auth/logout`, { method: 'POST' })
    expect(logoutResponse.status).toBe(204)
    expect(await logoutResponse.text()).toBe('')
    expect(await (await fetch(`${BASE_URL}/api/me`)).json()).toEqual(meUnauthenticated)
  })

  it('returns JSON 404 for logout in NeoShowcase mode', async () => {
    startMock('neoshowcase_logout_not_found')

    const response = await fetch(`${BASE_URL}/api/auth/logout`, {
      method: 'POST',
    })

    expect(response.status).toBe(404)
    expect(response.headers.get('content-type')).toContain('application/json')
    expect(await response.json()).toMatchObject({
      error: {
        code: 'MOCK_UNSPECIFIED',
        details: {},
      },
    })
  })

  it('returns JSON 400 when guest display_name is missing', async () => {
    startMock()

    const response = await fetch(`${BASE_URL}/api/auth/guest`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({}),
    })

    expect(response.status).toBe(400)
    expect(response.headers.get('content-type')).toContain('application/json')
    expect(await response.json()).toMatchObject({
      error: {
        code: 'MOCK_UNSPECIFIED',
        details: {},
      },
    })
  })

  it('validates guest display names by Unicode code point count', async () => {
    startMock()

    const emptyResponse = await fetch(`${BASE_URL}/api/auth/guest`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ display_name: '　  ' }),
    })
    expect(emptyResponse.status).toBe(422)
    expect(await emptyResponse.json()).toEqual(displayNameRequired)

    const longResponse = await fetch(`${BASE_URL}/api/auth/guest`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ display_name: '😀'.repeat(33) }),
    })
    expect(longResponse.status).toBe(422)
    expect(await longResponse.json()).toEqual(displayNameTooLong)
  })

  it('keeps authenticated state after guest display name validation errors', async () => {
    startMock()

    for (const displayName of ['　  ', '😀'.repeat(33)]) {
      const loginResponse = await fetch(`${BASE_URL}/api/auth/guest`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ display_name: displayName }),
      })

      expect(loginResponse.status).toBe(422)

      const meResponse = await fetch(`${BASE_URL}/api/me`)
      expect(meResponse.status).toBe(200)
      expect(await meResponse.json()).toEqual(meAuthenticated)
    }
  })

  it('returns the shared unauthorized fixture for unauthenticated game requests', async () => {
    startMock('game_api_unauthorized')

    const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/runs`, {
      method: 'POST',
    })
    expect(response.status).toBe(401)
    expect(await response.json()).toMatchObject({ error: { code: 'UNAUTHORIZED' } })
  })

  it('returns the shared locked-problem fixture', async () => {
    startMock('problem_locked')

    const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}`)
    expect(response.status).toBe(409)
    expect(await response.json()).toEqual(problemLocked)
  })

  it('returns presigned problem assets with no-store', async () => {
    startMock('get_problem_assets')

    const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/assets`)

    expect(response.status).toBe(200)
    expect(response.headers.get('cache-control')).toBe('no-store')
    expect(await response.json()).toEqual(problemAssets)
  })

  it.each([
    {
      scenarioId: 'get_problem_assets_unauthorized',
      expectedStatus: 401,
      expectedBody: unauthorized,
    },
    {
      scenarioId: 'get_problem_assets_without_active_run',
      expectedStatus: 404,
      expectedBody: runNotFound,
    },
    {
      scenarioId: 'get_problem_assets_not_found',
      expectedStatus: 404,
      expectedBody: imageNotFound,
    },
    {
      scenarioId: 'get_problem_assets_locked',
      expectedStatus: 409,
      expectedBody: problemLocked,
    },
  ])(
    'returns the shared fixture for $scenarioId',
    async ({ scenarioId, expectedStatus, expectedBody }) => {
      startMock(scenarioId)

      const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/assets`)

      expect(response.status).toBe(expectedStatus)
      expect(response.headers.get('content-type')).toContain('application/json')
      expect(await response.json()).toEqual(expectedBody)
    },
  )

  it('uses scenario judgments for query and answer submissions', async () => {
    startMock('query_correct')
    const queryResponse = await fetch(
      `${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/queries`,
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ source: 'serial', operations: [] }),
      },
    )
    expect(await queryResponse.json()).toEqual(queryCorrect)
    server?.close()

    startMock('answer_correct_and_clear_run')
    const answerResponse = await fetch(
      `${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/answers`,
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ answer: '19520715' }),
      },
    )
    expect(await answerResponse.json()).toEqual(answerCleared)
  })

  it('returns contract-shaped errors for invalid ids, missing resources, and long answers', async () => {
    const mock = startMock()

    const invalidIdResponse = await fetch(`${BASE_URL}/api/rooms/not-a-uuid/runs/current`)
    expect(invalidIdResponse.status).toBe(400)
    expect(await invalidIdResponse.json()).toMatchObject({ error: { details: {} } })

    mock.state.patch({ room_exists: false })
    const missingRoomResponse = await fetch(
      `${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}`,
    )
    expect(missingRoomResponse.status).toBe(404)

    mock.state.patch({ room_exists: true })
    const longAnswerResponse = await fetch(
      `${BASE_URL}/api/rooms/${ROOM_ID}/problems/${PROBLEM_ID}/answers`,
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ answer: 'x'.repeat(51) }),
      },
    )
    expect(longAnswerResponse.status).toBe(422)
    expect(await longAnswerResponse.json()).toMatchObject({ error: { details: {} } })
  })

  it('reproduces the invalid room id scenario when selected', async () => {
    startMock('invalid_room_id_format')

    const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/runs/current`)

    expect(response.status).toBe(400)
  })

  it('uses the response status declared by a scenario', async () => {
    const changedScenarioSource = scenarioSource.replace(
      /(id: start_new_run[\s\S]*?operation_id: startOrResumeRun[\s\S]*?status:) 200(\s+example:) new_run/,
      '$1 401$2 unauthorized',
    )
    const contract = createMockContract(openApiSource, changedScenarioSource, fixtureModules)
    startMock(undefined, contract)

    const response = await fetch(`${BASE_URL}/api/rooms/${ROOM_ID}/runs`, {
      method: 'POST',
    })

    expect(response.status).toBe(401)
    expect(await response.json()).toEqual(unauthorized)
  })
})

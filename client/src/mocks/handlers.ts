import { HttpResponse, type HttpHandler } from 'msw'
import { createOpenApiHttp } from 'openapi-msw'

import type { components, paths } from '@/generated/api'

import type { MockContract, MockScenarioStep } from './contract'
import { mockContract } from './data'
import { createMockState, type MockStateStore } from './state'

type Schemas = components['schemas']

export interface MockApiOptions {
  scenarioId?: string
  contract?: MockContract
}

export interface MockApi {
  handlers: HttpHandler[]
  state: MockStateStore
}

const http = createOpenApiHttp<paths>()
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function isUuid(value: string): boolean {
  return UUID_PATTERN.test(value)
}

function errorBody(status: number): Schemas['ErrorResponse'] {
  const messages: Record<number, string> = {
    400: 'リクエストの形式が正しくありません',
    404: '対象が見つかりません',
    422: '入力内容が正しくありません',
    500: 'サーバー内部でエラーが発生しました',
  }
  return {
    error: {
      code: 'MOCK_UNSPECIFIED',
      message: messages[status] ?? 'モックAPIでエラーが発生しました',
      details: {},
    },
  }
}

function responseExample<T>(
  contract: MockContract,
  operationId: string,
  status: number,
  example: string,
): T {
  return contract.getResponseExample(operationId, status, example) as T
}

interface ScenarioResponse<T> {
  status: number
  body: T
}

function responseFromStep<T>(
  contract: MockContract,
  state: MockStateStore,
  scenarioId: string,
  operationId: string,
): ScenarioResponse<T> {
  const step = state.applyStep(scenarioId, operationId)
  if (!step.response.example) {
    throw new Error(`scenario ${scenarioId} has no response example for ${operationId}`)
  }
  return {
    status: step.response.status,
    body: responseExample<T>(contract, operationId, step.response.status, step.response.example),
  }
}

function getStep(
  contract: MockContract,
  scenarioId: string,
  operationId: string,
): MockScenarioStep {
  const step = contract
    .getScenario(scenarioId)
    .steps.find((candidate) => candidate.operationId === operationId)
  if (!step) throw new Error(`scenario ${scenarioId} has no step for ${operationId}`)
  return step
}

function hasValidResourceIds(params: Record<string, string | readonly string[]>): boolean {
  return Object.values(params).every((value) =>
    typeof value === 'string' ? isUuid(value) : value.every(isUuid),
  )
}

export function createMockApi(options: MockApiOptions = {}): MockApi {
  const contract = options.contract ?? mockContract
  const state = createMockState(contract, options.scenarioId)
  const requestHasValidResourceIds = (params: Record<string, string | readonly string[]>) =>
    state.get('room_id_format') !== 'invalid_uuid' && hasValidResourceIds(params)

  const handlers = [
    http.get('/openapi.yaml', ({ response }) =>
      response.untyped(
        HttpResponse.text(contract.openApiSource, {
          headers: { 'content-type': 'application/yaml; charset=utf-8' },
        }),
      ),
    ),

    http.get('/api/me', () => {
      const scenarioId = `me_${String(state.get('auth_mode'))}_${
        state.get('authenticated') ? 'authenticated' : 'unauthenticated'
      }`
      const result = responseFromStep<Schemas['MeResponse']>(contract, state, scenarioId, 'getMe')
      return HttpResponse.json(result.body, { status: result.status })
    }),

    http.post('/api/auth/guest', async ({ request, response }) => {
      if (state.get('auth_mode') !== 'demo') {
        return response(404).json(errorBody(404))
      }

      let body: Schemas['GuestLoginRequest']
      try {
        body = await request.json()
      } catch {
        return response(400).json(errorBody(400))
      }
      if (typeof body.display_name !== 'string') {
        return response(400).json(errorBody(400))
      }

      const displayName = body.display_name.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, '')

      const displayNameLength = Array.from(displayName).length

      if (displayNameLength === 0) {
        const result = responseFromStep<Schemas['ErrorResponse']>(
          contract,
          state,
          'demo_login_display_name_required',
          'loginGuest',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }

      if (displayNameLength > 32) {
        const result = responseFromStep<Schemas['ErrorResponse']>(
          contract,
          state,
          'demo_login_display_name_too_long',
          'loginGuest',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }

      const result = responseFromStep<Schemas['GuestLoginResponse']>(
        contract,
        state,
        'demo_login_and_logout',
        'loginGuest',
      )
      return HttpResponse.json(result.body, { status: result.status })
    }),

    http.post('/api/auth/logout', ({ response }) => {
      if (state.get('auth_mode') !== 'demo') {
        return response(404).json(errorBody(404))
      }

      const step = state.applyStep('demo_login_and_logout', 'logoutDemo')
      return response.untyped(
        new HttpResponse(null, {
          status: step.response.status,
          headers: { 'content-length': '0' },
        }),
      )
    }),

    http.get('/api/rooms', () => {
      if (!state.get('room_exists')) {
        const result = responseFromStep<Schemas['RoomsResponse']>(
          contract,
          state,
          'rooms_empty',
          'getRooms',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }
      if (!state.get('authenticated')) {
        const result = responseFromStep<Schemas['RoomsResponse']>(
          contract,
          state,
          'rooms_unauthenticated',
          'getRooms',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }
      const scenarioId = state.get('active_run_exists')
        ? 'rooms_authenticated_active'
        : 'rooms_authenticated_cleared'
      const result = responseFromStep<Schemas['RoomsResponse']>(
        contract,
        state,
        scenarioId,
        'getRooms',
      )
      return HttpResponse.json(result.body, { status: result.status })
    }),

    http.post('/api/rooms/{room_id}/runs', ({ params, response }) => {
      if (!requestHasValidResourceIds(params)) return response(400).json(errorBody(400))
      if (!state.get('authenticated')) {
        return response(401).json(
          responseExample<Schemas['ErrorResponse']>(
            contract,
            'startOrResumeRun',
            401,
            'unauthorized',
          ),
        )
      }
      if (!state.get('room_exists')) return response(404).json(errorBody(404))

      const scenarioId = state.get('active_run_exists') ? 'resume_active_run' : 'start_new_run'
      const result = responseFromStep<Schemas['ActiveRunResponse'] | Schemas['ErrorResponse']>(
        contract,
        state,
        scenarioId,
        'startOrResumeRun',
      )
      return response.untyped(
        HttpResponse.json(result.body, {
          status: result.status,
        }),
      )
    }),

    http.get('/api/rooms/{room_id}/runs/current', ({ params, response }) => {
      if (!requestHasValidResourceIds(params)) return response(400).json(errorBody(400))
      if (!state.get('authenticated')) {
        return response(401).json(
          responseExample<Schemas['ErrorResponse']>(contract, 'getCurrentRun', 401, 'unauthorized'),
        )
      }
      if (!state.get('room_exists')) return response(404).json(errorBody(404))
      if (!state.get('active_run_exists')) {
        const result = responseFromStep<Schemas['ErrorResponse']>(
          contract,
          state,
          'current_run_not_found',
          'getCurrentRun',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }
      const result = responseFromStep<Schemas['ActiveRunResponse']>(
        contract,
        state,
        'get_current_run',
        'getCurrentRun',
      )
      return HttpResponse.json(result.body, { status: result.status })
    }),

    http.get('/api/rooms/{room_id}/problems/{problem_id}', ({ params, response }) => {
      if (!requestHasValidResourceIds(params)) return response(400).json(errorBody(400))
      if (!state.get('authenticated')) {
        return response(401).json(
          responseExample<Schemas['ErrorResponse']>(contract, 'getProblem', 401, 'unauthorized'),
        )
      }
      if (!state.get('room_exists') || !state.get('problem_exists')) {
        return response(404).json(errorBody(404))
      }
      if (state.get('problem_status') === 'locked') {
        const result = responseFromStep<Schemas['ErrorResponse']>(
          contract,
          state,
          'problem_locked',
          'getProblem',
        )
        return HttpResponse.json(result.body, { status: result.status })
      }
      const result = responseFromStep<Schemas['ProblemResponse']>(
        contract,
        state,
        'get_available_problem',
        'getProblem',
      )
      return HttpResponse.json(result.body, { status: result.status })
    }),

    http.post(
      '/api/rooms/{room_id}/problems/{problem_id}/queries',
      async ({ params, request, response }) => {
        if (!requestHasValidResourceIds(params)) return response(400).json(errorBody(400))
        if (!state.get('authenticated')) {
          return response(401).json(
            responseExample<Schemas['ErrorResponse']>(contract, 'submitQuery', 401, 'unauthorized'),
          )
        }
        if (!state.get('room_exists') || !state.get('problem_exists')) {
          return response(404).json(errorBody(404))
        }
        if (state.get('problem_status') === 'locked') {
          return response(409).json(
            responseExample<Schemas['ErrorResponse']>(
              contract,
              'submitQuery',
              409,
              'problem_locked',
            ),
          )
        }

        let body: Schemas['QueryRequest']
        try {
          body = await request.json()
        } catch {
          return response(400).json(errorBody(400))
        }
        if (typeof body.source !== 'string' || !Array.isArray(body.operations)) {
          return response(422).json(errorBody(422))
        }

        const scenarioId =
          state.get('query_judgement') === 'correct' ? 'query_correct' : 'query_incorrect'
        const result = responseFromStep<Schemas['QueryResponse']>(
          contract,
          state,
          scenarioId,
          'submitQuery',
        )
        return HttpResponse.json(result.body, { status: result.status })
      },
    ),

    http.post(
      '/api/rooms/{room_id}/problems/{problem_id}/answers',
      async ({ params, request, response }) => {
        if (!requestHasValidResourceIds(params)) return response(400).json(errorBody(400))
        if (!state.get('authenticated')) {
          return response(401).json(
            responseExample<Schemas['ErrorResponse']>(
              contract,
              'submitAnswer',
              401,
              'unauthorized',
            ),
          )
        }
        if (!state.get('room_exists') || !state.get('problem_exists')) {
          return response(404).json(errorBody(404))
        }
        if (state.get('problem_status') === 'locked') {
          return response(409).json(
            responseExample<Schemas['ErrorResponse']>(
              contract,
              'submitAnswer',
              409,
              'problem_locked',
            ),
          )
        }

        let body: Schemas['AnswerRequest']
        try {
          body = await request.json()
        } catch {
          return response(400).json(errorBody(400))
        }
        if (typeof body.answer !== 'string') return response(422).json(errorBody(422))

        const maxLength = Number(state.get('answer_max_length'))
        if (body.answer.length > maxLength) {
          const step = getStep(contract, 'answer_input_too_long', 'submitAnswer')
          state.patch(step.stateAfter)
          return response.untyped(
            HttpResponse.json(errorBody(step.response.status), {
              status: step.response.status,
            }),
          )
        }

        let scenarioId = 'answer_incorrect'
        if (state.get('answer_judgement') === 'correct') {
          scenarioId = state.get('last_required_problem')
            ? 'answer_correct_and_clear_run'
            : 'answer_correct_and_unlock'
        }
        const result = responseFromStep<Schemas['AnswerResponse']>(
          contract,
          state,
          scenarioId,
          'submitAnswer',
        )
        return HttpResponse.json(result.body, { status: result.status })
      },
    ),
  ]

  return { handlers, state }
}

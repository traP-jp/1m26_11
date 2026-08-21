import type { components, operations } from '@/generated/api'

type JsonContent<T> = T extends { content: { 'application/json': infer Body } } ? Body : never

export type GetMeResponse = JsonContent<operations['getMe']['responses'][200]>
export type LoginGuestRequest = JsonContent<operations['loginGuest']['requestBody']>
export type LoginGuestResponse = JsonContent<operations['loginGuest']['responses'][200]>
export type StartOrResumeRunPath = operations['startOrResumeRun']['parameters']['path']
export type StartOrResumeRunResponse = JsonContent<operations['startOrResumeRun']['responses'][200]>
export type GetCurrentRunPath = operations['getCurrentRun']['parameters']['path']
export type GetCurrentRunResponse = JsonContent<operations['getCurrentRun']['responses'][200]>
export type GetProblemPath = operations['getProblem']['parameters']['path']
export type GetProblemResponse = JsonContent<operations['getProblem']['responses'][200]>
export type SubmitQueryPath = operations['submitQuery']['parameters']['path']
export type SubmitQueryRequest = JsonContent<operations['submitQuery']['requestBody']>
export type SubmitQueryResponse = JsonContent<operations['submitQuery']['responses'][200]>
export type SubmitAnswerPath = operations['submitAnswer']['parameters']['path']
export type SubmitAnswerRequest = JsonContent<operations['submitAnswer']['requestBody']>
export type SubmitAnswerResponse = JsonContent<operations['submitAnswer']['responses'][200]>

type ApiErrorResponse = components['schemas']['ErrorResponse']
export type ApiErrorDetails = ApiErrorResponse['error']['details']

export type ApiClientFailure =
  | {
      kind: 'http'
      status: number
      code: string
      details: ApiErrorDetails
    }
  | {
      kind: 'network'
      cause: unknown
    }
  | {
      kind: 'invalid-response'
      status: number
      cause?: unknown
    }

export type ApiClientErrorKind = ApiClientFailure['kind']

export class ApiClientError extends Error {
  override readonly name = 'ApiClientError'
  readonly kind: ApiClientErrorKind
  readonly status: number | undefined
  readonly code: string | undefined
  readonly details: ApiErrorDetails | undefined
  readonly cause: unknown

  constructor(
    message: string,
    readonly failure: ApiClientFailure,
  ) {
    super(message)
    this.kind = failure.kind
    this.status = 'status' in failure ? failure.status : undefined
    this.code = failure.kind === 'http' ? failure.code : undefined
    this.details = failure.kind === 'http' ? failure.details : undefined
    this.cause = 'cause' in failure ? failure.cause : undefined
  }
}

export interface ApiClient {
  getMe(): Promise<GetMeResponse>
  loginGuest(body: LoginGuestRequest): Promise<LoginGuestResponse>
  logoutDemo(): Promise<void>
  startOrResumeRun(path: StartOrResumeRunPath): Promise<StartOrResumeRunResponse>
  getCurrentRun(path: GetCurrentRunPath): Promise<GetCurrentRunResponse>
  getProblem(path: GetProblemPath): Promise<GetProblemResponse>
  submitQuery(path: SubmitQueryPath, body: SubmitQueryRequest): Promise<SubmitQueryResponse>
  submitAnswer(path: SubmitAnswerPath, body: SubmitAnswerRequest): Promise<SubmitAnswerResponse>
}

export interface CreateApiClientOptions {
  baseUrl?: string
  fetch?: typeof globalThis.fetch
}

type HttpMethod = 'GET' | 'POST'

interface RequestOptions {
  method: HttpMethod
  body?: unknown
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function isApiErrorResponse(value: unknown): value is ApiErrorResponse {
  if (!isRecord(value) || !isRecord(value.error)) return false

  return (
    typeof value.error.code === 'string' &&
    typeof value.error.message === 'string' &&
    isRecord(value.error.details)
  )
}

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, '')
}

function invalidResponse(message: string, status: number, cause?: unknown): ApiClientError {
  return new ApiClientError(message, {
    kind: 'invalid-response',
    status,
    cause,
  })
}

async function parseJson(response: Response): Promise<unknown> {
  try {
    return await response.json()
  } catch (cause) {
    throw invalidResponse('API responseをJSONとして読み取れませんでした', response.status, cause)
  }
}

async function throwResponseError(response: Response): Promise<never> {
  const body = await parseJson(response)
  if (!isApiErrorResponse(body)) {
    throw invalidResponse('API error responseの形式が正しくありません', response.status)
  }

  throw new ApiClientError(body.error.message, {
    kind: 'http',
    status: response.status,
    code: body.error.code,
    details: body.error.details,
  })
}

export function createApiClient(options: CreateApiClientOptions = {}): ApiClient {
  const baseUrl = normalizeBaseUrl(options.baseUrl ?? globalThis.location?.origin ?? '')
  const fetchApi = options.fetch

  async function send(path: string, options: RequestOptions): Promise<Response> {
    const hasBody = options.body !== undefined
    const headers: Record<string, string> = { accept: 'application/json' }
    if (hasBody) headers['content-type'] = 'application/json'

    try {
      return await (fetchApi ?? globalThis.fetch)(`${baseUrl}${path}`, {
        method: options.method,
        credentials: 'include',
        headers,
        body: hasBody ? JSON.stringify(options.body) : undefined,
      })
    } catch (cause) {
      throw new ApiClientError('APIとの通信に失敗しました', {
        kind: 'network',
        cause,
      })
    }
  }

  async function requestJson<T>(
    path: string,
    options: RequestOptions,
    expectedStatus = 200,
  ): Promise<T> {
    const response = await send(path, options)
    if (!response.ok) return throwResponseError(response)
    if (response.status !== expectedStatus) {
      throw invalidResponse(
        `API responseのstatusが不正です（expected: ${expectedStatus}, actual: ${response.status}）`,
        response.status,
      )
    }

    return (await parseJson(response)) as T
  }

  async function requestNoContent(path: string, options: RequestOptions): Promise<void> {
    const response = await send(path, options)
    if (!response.ok) return throwResponseError(response)
    if (response.status !== 204) {
      throw invalidResponse(
        `API responseのstatusが不正です（expected: 204, actual: ${response.status}）`,
        response.status,
      )
    }
  }

  return {
    getMe: () => requestJson<GetMeResponse>('/api/me', { method: 'GET' }),

    loginGuest: (body) =>
      requestJson<LoginGuestResponse>('/api/auth/guest', { method: 'POST', body }),

    logoutDemo: () => requestNoContent('/api/auth/logout', { method: 'POST' }),

    startOrResumeRun: ({ room_id }) =>
      requestJson<StartOrResumeRunResponse>(`/api/rooms/${encodeURIComponent(room_id)}/runs`, {
        method: 'POST',
      }),

    getCurrentRun: ({ room_id }) =>
      requestJson<GetCurrentRunResponse>(`/api/rooms/${encodeURIComponent(room_id)}/runs/current`, {
        method: 'GET',
      }),

    getProblem: ({ room_id, problem_id }) =>
      requestJson<GetProblemResponse>(
        `/api/rooms/${encodeURIComponent(room_id)}/problems/${encodeURIComponent(problem_id)}`,
        { method: 'GET' },
      ),

    submitQuery: ({ room_id, problem_id }, body) =>
      requestJson<SubmitQueryResponse>(
        `/api/rooms/${encodeURIComponent(room_id)}/problems/${encodeURIComponent(problem_id)}/queries`,
        { method: 'POST', body },
      ),

    submitAnswer: ({ room_id, problem_id }, body) =>
      requestJson<SubmitAnswerResponse>(
        `/api/rooms/${encodeURIComponent(room_id)}/problems/${encodeURIComponent(problem_id)}/answers`,
        { method: 'POST', body },
      ),
  }
}

export const apiClient = createApiClient()

import { describe, expect, it, vi } from 'vitest'

import meAuthenticated from '../../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../../openapi/examples/auth/me-demo-unauthenticated.json'
import guestResponse from '../../../../openapi/examples/auth/guest-response.json'
import type { ApiClient, GetMeResponse, LoginGuestResponse } from '@/api/client'
import { authStateFromMe, createAuthFlow } from '@/utils/auth'

const authenticatedFixture = meAuthenticated as GetMeResponse
const unauthenticatedFixture = meUnauthenticated as GetMeResponse
const guestFixture = guestResponse as LoginGuestResponse

function createClient(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(unauthenticatedFixture),
    loginGuest: vi.fn<ApiClient['loginGuest']>().mockResolvedValue(guestFixture),
    logoutDemo: vi.fn<ApiClient['logoutDemo']>().mockResolvedValue(undefined),
    startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
    getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
    getProblem: vi.fn<ApiClient['getProblem']>(),
    submitQuery: vi.fn<ApiClient['submitQuery']>(),
    submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
    ...overrides,
  }
}

function deferred<T>(): { promise: Promise<T>; resolve(value: T): void } {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

describe('auth state', () => {
  it('maps the shared me fixtures without exposing the internal user id', () => {
    expect(authStateFromMe(meUnauthenticated as GetMeResponse)).toEqual({
      status: 'unauthenticated',
      authMode: 'demo',
      loginUrl: null,
      busy: false,
      error: null,
    })
    expect(authStateFromMe(meAuthenticated as GetMeResponse)).toEqual({
      status: 'authenticated',
      authMode: 'demo',
      displayName: 'kaomojikun',
      logoutUrl: null,
      busy: false,
      error: null,
    })
  })

  it('decides the final guest login state by fetching me again', async () => {
    const client = createClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(unauthenticatedFixture)
        .mockResolvedValueOnce(authenticatedFixture),
    })
    const auth = createAuthFlow(client)

    await auth.refresh()
    await auth.loginGuest('kaomojikun')

    expect(client.loginGuest).toHaveBeenCalledExactlyOnceWith({ display_name: 'kaomojikun' })
    expect(client.getMe).toHaveBeenCalledTimes(2)
    expect(auth.state.value.status).toBe('authenticated')
  })

  it('decides the final logout state by fetching me again', async () => {
    const client = createClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(authenticatedFixture)
        .mockResolvedValueOnce(unauthenticatedFixture),
    })
    const auth = createAuthFlow(client)

    await auth.refresh()
    await auth.logout()

    expect(client.logoutDemo).toHaveBeenCalledOnce()
    expect(client.getMe).toHaveBeenCalledTimes(2)
    expect(auth.state.value.status).toBe('unauthenticated')
  })

  it('keeps busy and prevents duplicate guest submissions', async () => {
    const pending = deferred<LoginGuestResponse>()
    const client = createClient({
      loginGuest: vi.fn<ApiClient['loginGuest']>(() => pending.promise),
    })
    const auth = createAuthFlow(client)
    await auth.refresh()

    const first = auth.loginGuest('kaomojikun')
    const second = auth.loginGuest('duplicated')

    expect(auth.state.value).toMatchObject({ status: 'unauthenticated', busy: true })
    expect(client.loginGuest).toHaveBeenCalledOnce()
    pending.resolve(guestFixture)
    await Promise.all([first, second])
  })

  it('retains the current state and error when an operation fails', async () => {
    const error = new Error('guest failed')
    const client = createClient({
      loginGuest: vi.fn<ApiClient['loginGuest']>().mockRejectedValue(error),
    })
    const auth = createAuthFlow(client)
    await auth.refresh()

    await auth.loginGuest('kaomojikun')

    expect(auth.state.value).toMatchObject({
      status: 'unauthenticated',
      busy: false,
      error,
    })
    expect(client.getMe).toHaveBeenCalledOnce()
  })

  it('retries only me when refreshing after a successful guest login', async () => {
    const refreshError = new Error('me failed after login')
    const client = createClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(unauthenticatedFixture)
        .mockRejectedValueOnce(refreshError)
        .mockResolvedValueOnce(authenticatedFixture),
    })
    const auth = createAuthFlow(client)
    await auth.refresh()

    await auth.loginGuest('kaomojikun')

    expect(auth.state.value).toEqual({ status: 'error', error: refreshError })
    await auth.refresh()
    expect(auth.state.value.status).toBe('authenticated')
    expect(client.loginGuest).toHaveBeenCalledOnce()
    expect(client.getMe).toHaveBeenCalledTimes(3)
  })

  it('retries only me when refreshing after a successful logout', async () => {
    const refreshError = new Error('me failed after logout')
    const client = createClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(authenticatedFixture)
        .mockRejectedValueOnce(refreshError)
        .mockResolvedValueOnce(unauthenticatedFixture),
    })
    const auth = createAuthFlow(client)
    await auth.refresh()

    await auth.logout()

    expect(auth.state.value).toEqual({ status: 'error', error: refreshError })
    await auth.refresh()
    expect(auth.state.value.status).toBe('unauthenticated')
    expect(client.logoutDemo).toHaveBeenCalledOnce()
    expect(client.getMe).toHaveBeenCalledTimes(3)
  })

  it('moves to error when the initial me request fails', async () => {
    const error = new Error('me failed')
    const auth = createAuthFlow(
      createClient({ getMe: vi.fn<ApiClient['getMe']>().mockRejectedValue(error) }),
    )

    await auth.refresh()

    expect(auth.state.value).toEqual({ status: 'error', error })
  })
})

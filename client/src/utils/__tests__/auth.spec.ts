import { describe, expect, it, vi } from 'vitest'

import meAuthenticated from '../../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../../openapi/examples/auth/me-demo-unauthenticated.json'
import meNeoshowcaseAuthenticated from '../../../../openapi/examples/auth/me-neoshowcase-authenticated.json'
import meNeoshowcaseUnauthenticated from '../../../../openapi/examples/auth/me-neoshowcase-unauthenticated.json'
import guestResponse from '../../../../openapi/examples/auth/guest-response.json'
import type { ApiClient, GetMeResponse, LoginGuestResponse } from '@/api/client'
import {
  authStateFromMe,
  createAuthController,
  createAuthNavigationRequest,
  type AuthNavigationHandler,
} from '@/utils/auth'

const authenticatedFixture = meAuthenticated as GetMeResponse
const unauthenticatedFixture = meUnauthenticated as GetMeResponse
const neoshowcaseAuthenticatedFixture = meNeoshowcaseAuthenticated as GetMeResponse
const neoshowcaseUnauthenticatedFixture = meNeoshowcaseUnauthenticated as GetMeResponse
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
    createProblem: vi.fn<ApiClient['createProblem']>(),
    uploadProblemAsset: vi.fn<ApiClient['uploadProblemAsset']>(),
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
  it('maps the shared Demo me fixtures without exposing the internal user id', () => {
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

  it('maps the shared NeoShowcase me fixtures and keeps the API navigation URLs', () => {
    expect(authStateFromMe(neoshowcaseUnauthenticatedFixture)).toEqual({
      status: 'unauthenticated',
      authMode: 'neoshowcase',
      loginUrl: '/_oauth/login?redirect=/',
      busy: false,
      error: null,
    })
    expect(authStateFromMe(neoshowcaseAuthenticatedFixture)).toEqual({
      status: 'authenticated',
      authMode: 'neoshowcase',
      displayName: 'kaomojikun',
      logoutUrl: '/_oauth/logout?redirect=/',
      busy: false,
      error: null,
    })
  })

  it.each([
    '',
    '  ',
    'javascript:alert(1)',
    'data:text/html,test',
    'blob:https://game.trap.show/fixture',
    'http://auth.trap.jp/login',
    'https://user:password@auth.trap.jp/login',
    'http://[',
  ])('rejects an unsafe authentication navigation URL: %s', (url) => {
    expect(() => createAuthNavigationRequest(url, 'https://game.trap.show/')).toThrow(
      '認証用の遷移URLが不正です',
    )
  })

  it.each([
    ['/_oauth/login?redirect=/', 'https://game.trap.show/'],
    ['https://auth.trap.jp/login?redirect=/rooms/1', 'https://game.trap.show/'],
    ['http://localhost:5173/_oauth/login', 'http://localhost:5173/'],
  ])('keeps an allowed authentication navigation URL unchanged', (url, currentUrl) => {
    expect(createAuthNavigationRequest(url, currentUrl)).toEqual({ type: 'navigate', url })
  })

  it.each([
    { ...meNeoshowcaseUnauthenticated, login_url: null },
    { ...meNeoshowcaseUnauthenticated, logout_url: '/unexpected' },
    { ...meNeoshowcaseAuthenticated, login_url: '/unexpected' },
    { ...meNeoshowcaseAuthenticated, logout_url: 'javascript:alert(1)' },
    { ...meUnauthenticated, login_url: '/unexpected' },
    { ...meAuthenticated, logout_url: '/unexpected' },
    { ...meUnauthenticated, auth_mode: 'unknown' },
    { ...meNeoshowcaseAuthenticated, authenticated: 'true' },
    { ...meNeoshowcaseUnauthenticated, user: meNeoshowcaseAuthenticated.user },
    { ...meNeoshowcaseAuthenticated, user: null },
    {
      ...meNeoshowcaseAuthenticated,
      user: { ...meNeoshowcaseAuthenticated.user, display_name: 1 },
    },
  ])('rejects an inconsistent me response before exposing it to the UI', (response) => {
    expect(() => authStateFromMe(response as unknown as GetMeResponse)).toThrow(Error)
  })

  it('decides the final guest login state by fetching me again', async () => {
    const client = createClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(unauthenticatedFixture)
        .mockResolvedValueOnce(authenticatedFixture),
    })
    const auth = createAuthController(client)

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
    const auth = createAuthController(client)

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
    const auth = createAuthController(client)
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
    const auth = createAuthController(client)
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
    const auth = createAuthController(client)
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
    const auth = createAuthController(client)
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
    const auth = createAuthController(
      createClient({ getMe: vi.fn<ApiClient['getMe']>().mockRejectedValue(error) }),
    )

    await auth.refresh()

    expect(auth.state.value).toEqual({ status: 'error', error })
  })

  it('turns an unsafe NeoShowcase URL into the existing auth error state', async () => {
    const auth = createAuthController(
      createClient({
        getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue({
          ...meNeoshowcaseUnauthenticated,
          login_url: 'javascript:alert(1)',
        } as unknown as GetMeResponse),
      }),
    )

    await auth.refresh()

    expect(auth.state.value.status).toBe('error')
    expect(auth.portalUserStatus.value).toBeNull()
  })

  it('requests the API-provided NeoShowcase login URL once and keeps busy until navigation', async () => {
    const navigate = vi.fn<AuthNavigationHandler>()
    const client = createClient({
      getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(neoshowcaseUnauthenticatedFixture),
    })
    const auth = createAuthController(client, navigate)
    await auth.refresh()

    await Promise.all([auth.login(), auth.login()])

    expect(navigate).toHaveBeenCalledExactlyOnceWith({
      type: 'navigate',
      url: '/_oauth/login?redirect=/',
    })
    expect(auth.state.value).toMatchObject({
      status: 'unauthenticated',
      busy: true,
      error: null,
    })
    expect(client.loginGuest).not.toHaveBeenCalled()
  })

  it('requests the API-provided NeoShowcase logout URL without calling the Demo API', async () => {
    const navigate = vi.fn<AuthNavigationHandler>()
    const client = createClient({
      getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(neoshowcaseAuthenticatedFixture),
    })
    const auth = createAuthController(client, navigate)
    await auth.refresh()

    await Promise.all([auth.logout(), auth.logout()])

    expect(navigate).toHaveBeenCalledExactlyOnceWith({
      type: 'navigate',
      url: '/_oauth/logout?redirect=/',
    })
    expect(auth.state.value).toMatchObject({ status: 'authenticated', busy: true, error: null })
    expect(client.logoutDemo).not.toHaveBeenCalled()
  })

  it('restores the current state with an error when normal navigation cannot start', async () => {
    const error = new Error('navigation failed')
    const navigate = vi.fn<AuthNavigationHandler>(() => {
      throw error
    })
    const auth = createAuthController(
      createClient({
        getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(neoshowcaseUnauthenticatedFixture),
      }),
      navigate,
    )
    await auth.refresh()

    await auth.login()

    expect(auth.state.value).toMatchObject({
      status: 'unauthenticated',
      busy: false,
      error,
    })
  })
})

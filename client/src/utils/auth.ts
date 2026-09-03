import {
  computed,
  readonly,
  ref,
  type ComputedRef,
  type DeepReadonly,
  type InjectionKey,
  type Ref,
} from 'vue'

import { apiClient, type ApiClient, type GetMeResponse, type LoginGuestRequest } from '@/api/client'
import type { PortalUserStatusState } from '@/components/portal/PortalHeader.types'

type AuthenticatedMe = Extract<GetMeResponse, { authenticated: true }>
type UnauthenticatedMe = Extract<GetMeResponse, { authenticated: false }>

export type AuthState =
  | { status: 'loading' }
  | {
      status: 'unauthenticated'
      authMode: UnauthenticatedMe['auth_mode']
      loginUrl: string | null
      busy: boolean
      error: unknown | null
    }
  | {
      status: 'authenticated'
      authMode: AuthenticatedMe['auth_mode']
      displayName: string
      logoutUrl: string | null
      busy: boolean
      error: unknown | null
    }
  | { status: 'error'; error: unknown }

export interface AuthNavigationRequest {
  type: 'navigate'
  url: string
}

export type AuthNavigationHandler = (request: AuthNavigationRequest) => void

function invalidAuthNavigationUrl(): Error {
  return new Error('認証用の遷移URLが不正です')
}

export function createAuthNavigationRequest(
  value: unknown,
  currentUrl = globalThis.location?.href ?? 'https://localhost/',
): AuthNavigationRequest {
  if (typeof value !== 'string' || value.length === 0 || value.trim() !== value) {
    throw invalidAuthNavigationUrl()
  }

  let current: URL
  let destination: URL
  try {
    current = new URL(currentUrl)
    destination = new URL(value, current)
  } catch {
    throw invalidAuthNavigationUrl()
  }

  const isHttps = destination.protocol === 'https:'
  const isCurrentHttpOrigin =
    destination.protocol === 'http:' && destination.origin === current.origin
  if (
    (!isHttps && !isCurrentHttpOrigin) ||
    destination.username !== '' ||
    destination.password !== ''
  ) {
    throw invalidAuthNavigationUrl()
  }

  return { type: 'navigate', url: value }
}

export function authStateFromMe(me: GetMeResponse): AuthState {
  if (me.authenticated !== true && me.authenticated !== false) {
    throw new Error('認証状態が不正です')
  }
  if (me.auth_mode !== 'demo' && me.auth_mode !== 'neoshowcase') {
    throw new Error('未対応の認証modeです')
  }

  if (me.authenticated === false) {
    if (me.user !== null) throw new Error('未認証userが不正です')
    if (me.auth_mode === 'demo' && (me.login_url !== null || me.logout_url !== null)) {
      throw new Error('Demo認証の遷移URLが不正です')
    }
    if (me.auth_mode === 'neoshowcase' && me.logout_url !== null) {
      throw new Error('NeoShowcase認証の遷移URLが不正です')
    }

    return {
      status: 'unauthenticated',
      authMode: me.auth_mode,
      loginUrl:
        me.auth_mode === 'neoshowcase' ? createAuthNavigationRequest(me.login_url).url : null,
      busy: false,
      error: null,
    }
  }

  if (me.user === null || typeof me.user.display_name !== 'string') {
    throw new Error('認証済みuserが不正です')
  }
  if (me.auth_mode === 'demo' && (me.login_url !== null || me.logout_url !== null)) {
    throw new Error('Demo認証の遷移URLが不正です')
  }
  if (me.auth_mode === 'neoshowcase' && me.login_url !== null) {
    throw new Error('NeoShowcase認証の遷移URLが不正です')
  }

  return {
    status: 'authenticated',
    authMode: me.auth_mode,
    displayName: me.user.display_name,
    logoutUrl:
      me.auth_mode === 'neoshowcase' ? createAuthNavigationRequest(me.logout_url).url : null,
    busy: false,
    error: null,
  }
}

export function portalUserStatusFromMe(me: GetMeResponse, pending = false): PortalUserStatusState {
  const state = authStateFromMe(me)
  if (state.status === 'unauthenticated') {
    return {
      authenticated: false,
      authMode: state.authMode,
      loginHref: state.loginUrl,
      loginPending: pending,
    }
  }

  if (state.status !== 'authenticated') throw new Error('me fixtureから認証状態を作成できません')

  return {
    authenticated: true,
    authMode: state.authMode,
    displayName: state.displayName,
    logoutHref: state.logoutUrl,
    logoutPending: pending,
  }
}

export function portalUserStatusFromAuthState(state: AuthState): PortalUserStatusState | null {
  if (state.status === 'unauthenticated') {
    return {
      authenticated: false,
      authMode: state.authMode,
      loginHref: state.loginUrl,
      loginPending: state.busy,
    }
  }
  if (state.status === 'authenticated') {
    return {
      authenticated: true,
      authMode: state.authMode,
      displayName: state.displayName,
      logoutHref: state.logoutUrl,
      logoutPending: state.busy,
    }
  }
  return null
}

export interface AuthController {
  state: DeepReadonly<Ref<AuthState>>
  portalUserStatus: ComputedRef<PortalUserStatusState | null>
  refresh(): Promise<void>
  login(): Promise<void>
  loginGuest(displayName: LoginGuestRequest['display_name']): Promise<void>
  logout(): Promise<void>
}

export const authApiClientKey: InjectionKey<ApiClient> = Symbol('authApiClient')
export const authNavigationHandlerKey: InjectionKey<AuthNavigationHandler> =
  Symbol('authNavigationHandler')

export function createAuthController(
  client: ApiClient = apiClient,
  navigate: AuthNavigationHandler = ({ url }) => globalThis.location.assign(url),
): AuthController {
  const state = ref<AuthState>({ status: 'loading' })
  let operationPending = false

  async function fetchMe(): Promise<void> {
    state.value = authStateFromMe(await client.getMe())
  }

  async function refresh(): Promise<void> {
    if (operationPending) return
    state.value = { status: 'loading' }
    try {
      await fetchMe()
    } catch (error) {
      state.value = { status: 'error', error }
    }
  }

  async function runOperation(operation: () => Promise<unknown>): Promise<void> {
    if (operationPending) return
    const current = state.value
    if (current.status !== 'authenticated' && current.status !== 'unauthenticated') return

    operationPending = true
    state.value = { ...current, busy: true, error: null }
    try {
      await operation()
    } catch (error) {
      state.value = { ...current, busy: false, error }
      operationPending = false
      return
    }

    try {
      await fetchMe()
    } catch (error) {
      state.value = { status: 'error', error }
    } finally {
      operationPending = false
    }
  }

  async function loginGuest(displayName: LoginGuestRequest['display_name']): Promise<void> {
    if (state.value.status !== 'unauthenticated' || state.value.authMode !== 'demo') return
    await runOperation(() => client.loginGuest({ display_name: displayName }))
  }

  function runNavigation(url: string | null): void {
    if (operationPending) return
    const current = state.value
    if (current.status !== 'authenticated' && current.status !== 'unauthenticated') return

    operationPending = true
    state.value = { ...current, busy: true, error: null }
    try {
      navigate(createAuthNavigationRequest(url))
    } catch (error) {
      state.value = { ...current, busy: false, error }
      operationPending = false
    }
  }

  async function login(): Promise<void> {
    if (state.value.status !== 'unauthenticated' || state.value.authMode !== 'neoshowcase') return
    runNavigation(state.value.loginUrl)
  }

  async function logout(): Promise<void> {
    if (state.value.status !== 'authenticated') return
    if (state.value.authMode === 'neoshowcase') {
      runNavigation(state.value.logoutUrl)
      return
    }
    await runOperation(() => client.logoutDemo())
  }

  return {
    state: readonly(state),
    portalUserStatus: computed(() => portalUserStatusFromAuthState(state.value)),
    refresh,
    login,
    loginGuest,
    logout,
  }
}

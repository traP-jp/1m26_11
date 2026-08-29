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

export function authStateFromMe(me: GetMeResponse): AuthState {
  if (!me.authenticated) {
    return {
      status: 'unauthenticated',
      authMode: me.auth_mode,
      loginUrl: me.login_url,
      busy: false,
      error: null,
    }
  }

  return {
    status: 'authenticated',
    authMode: me.auth_mode,
    displayName: me.user.display_name,
    logoutUrl: me.logout_url,
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

export interface AuthFlow {
  state: DeepReadonly<Ref<AuthState>>
  portalUserStatus: ComputedRef<PortalUserStatusState | null>
  refresh(): Promise<void>
  loginGuest(displayName: LoginGuestRequest['display_name']): Promise<void>
  logout(): Promise<void>
}

export const authApiClientKey: InjectionKey<ApiClient> = Symbol('authApiClient')

export function createAuthFlow(client: ApiClient = apiClient): AuthFlow {
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
      await fetchMe()
    } catch (error) {
      const latest = state.value
      if (latest.status === 'authenticated' || latest.status === 'unauthenticated') {
        state.value = { ...latest, busy: false, error }
      } else {
        state.value = { status: 'error', error }
      }
    } finally {
      operationPending = false
    }
  }

  async function loginGuest(displayName: LoginGuestRequest['display_name']): Promise<void> {
    if (state.value.status !== 'unauthenticated' || state.value.authMode !== 'demo') return
    await runOperation(() => client.loginGuest({ display_name: displayName }))
  }

  async function logout(): Promise<void> {
    if (state.value.status !== 'authenticated' || state.value.authMode !== 'demo') return
    await runOperation(() => client.logoutDemo())
  }

  return {
    state: readonly(state),
    portalUserStatus: computed(() => portalUserStatusFromAuthState(state.value)),
    refresh,
    loginGuest,
    logout,
  }
}

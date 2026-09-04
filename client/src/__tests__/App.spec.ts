import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, type Router } from 'vue-router'

import App from '../App.vue'
import PortalPage from '../PortalPage.vue'
import RoomPage from '../RoomPage.vue'
import AuthActionButton from '../components/auth/AuthActionButton.vue'
import UserMenu from '../components/auth/UserMenu.vue'
import AuthorProblemPage from '../AuthorProblemPage.vue'
import { createAppRouter } from '../router'
import guestResponse from '../../../openapi/examples/auth/guest-response.json'
import meAuthenticated from '../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../openapi/examples/auth/me-demo-unauthenticated.json'
import meNeoshowcaseAuthenticated from '../../../openapi/examples/auth/me-neoshowcase-authenticated.json'
import meNeoshowcaseUnauthenticated from '../../../openapi/examples/auth/me-neoshowcase-unauthenticated.json'
import type { ApiClient, GetMeResponse, LoginGuestResponse } from '../api/client'
import {
  authApiClientKey,
  authNavigationHandlerKey,
  type AuthNavigationHandler,
} from '../utils/auth'

const apiClient: ApiClient = {
  getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(meAuthenticated as GetMeResponse),
  loginGuest: vi.fn<ApiClient['loginGuest']>(),
  logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
  startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
  getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
  getProblem: vi.fn<ApiClient['getProblem']>(),
  submitQuery: vi.fn<ApiClient['submitQuery']>(),
  submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
}

async function mountAt(
  path: string,
  client: ApiClient = apiClient,
  navigate: AuthNavigationHandler = () => undefined,
): Promise<{ router: Router; wrapper: ReturnType<typeof mount> }> {
  const router = createAppRouter(createMemoryHistory())
  await router.push(path)
  const wrapper = mount(App, {
    global: {
      plugins: [router],
      provide: {
        [authApiClientKey as symbol]: client,
        [authNavigationHandlerKey as symbol]: navigate,
      },
    },
  })
  await router.isReady()
  await flushPromises()
  return { router, wrapper }
}

describe('App', () => {
  it('renders the portal page at the root route', async () => {
    const { wrapper } = await mountAt('/')

    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('falls back to the portal page for an unknown route', async () => {
    const { router, wrapper } = await mountAt('/unknown')

    expect(router.currentRoute.value.name).toBe('portal')
    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('navigates from Portal to Room and back through semantic UI events', async () => {
    const { router, wrapper } = await mountAt('/')

    wrapper.getComponent(PortalPage).vm.$emit('startRoom', '1411824c-d357-4941-af76-c76cb827dda6')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/rooms/1411824c-d357-4941-af76-c76cb827dda6')
    expect(wrapper.findComponent(RoomPage).exists()).toBe(true)

    wrapper.getComponent(RoomPage).vm.$emit('uiEvent', { type: 'room-exited' })
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/')
    expect(wrapper.findComponent(PortalPage).exists()).toBe(true)
  })

  it('renders the development problem authoring page without changing App composition', async () => {
    const roomId = '1411824c-d357-4941-af76-c76cb827dda6'
    const { router, wrapper } = await mountAt(`/author/rooms/${roomId}/problems/new`)

    expect(router.currentRoute.value.name).toBe('problem-author-new')
    expect(wrapper.getComponent(AuthorProblemPage).props('roomId')).toBe(roomId)
  })

  it('connects Demo guest login to the auth flow', async () => {
    const guestClient: ApiClient = {
      ...apiClient,
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(meUnauthenticated as GetMeResponse)
        .mockResolvedValueOnce(meAuthenticated as GetMeResponse),
      loginGuest: vi
        .fn<ApiClient['loginGuest']>()
        .mockResolvedValue(guestResponse as LoginGuestResponse),
    }
    const { wrapper } = await mountAt('/', guestClient)

    wrapper.getComponent(PortalPage).vm.$emit('guestLogin', 'kaomojikun')
    await flushPromises()

    expect(guestClient.loginGuest).toHaveBeenCalledExactlyOnceWith({
      display_name: 'kaomojikun',
    })
    expect(guestClient.getMe).toHaveBeenCalledTimes(2)
    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('routes a NeoShowcase login control through the auth controller', async () => {
    const navigate = vi.fn<AuthNavigationHandler>()
    const neoClient: ApiClient = {
      ...apiClient,
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValue(meNeoshowcaseUnauthenticated as GetMeResponse),
      loginGuest: vi.fn<ApiClient['loginGuest']>(),
      logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    }
    const { wrapper } = await mountAt('/', neoClient, navigate)

    await wrapper.getComponent(AuthActionButton).trigger('click')
    await flushPromises()

    expect(navigate).toHaveBeenCalledExactlyOnceWith({
      type: 'navigate',
      url: '/_oauth/login?redirect=/',
    })
    expect(neoClient.loginGuest).not.toHaveBeenCalled()
    expect(wrapper.getComponent(PortalPage).props('authBusy')).toBe(true)
    expect(wrapper.getComponent(AuthActionButton).element.tagName).toBe('BUTTON')
    expect(wrapper.getComponent(AuthActionButton).attributes('disabled')).toBeDefined()
  })

  it('routes a NeoShowcase logout control without calling the Demo API', async () => {
    const navigate = vi.fn<AuthNavigationHandler>()
    const neoClient: ApiClient = {
      ...apiClient,
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValue(meNeoshowcaseAuthenticated as GetMeResponse),
      loginGuest: vi.fn<ApiClient['loginGuest']>(),
      logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    }
    const { wrapper } = await mountAt('/', neoClient, navigate)
    const userMenu = wrapper.getComponent(UserMenu)

    await userMenu.get('button').trigger('click')
    await userMenu.get('a').trigger('click')
    await flushPromises()

    expect(navigate).toHaveBeenCalledExactlyOnceWith({
      type: 'navigate',
      url: '/_oauth/logout?redirect=/',
    })
    expect(neoClient.logoutDemo).not.toHaveBeenCalled()
    expect(wrapper.getComponent(PortalPage).props('authBusy')).toBe(true)
  })

  it('retries the initial auth request after an error', async () => {
    const retryClient: ApiClient = {
      ...apiClient,
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockRejectedValueOnce(new Error('me failed'))
        .mockResolvedValueOnce(meAuthenticated as GetMeResponse),
    }
    const { wrapper } = await mountAt('/', retryClient)

    expect(wrapper.get('[role="alert"]').text()).toContain('認証状態を取得できませんでした。')
    await wrapper.get('[role="alert"] button').trigger('click')
    await flushPromises()

    expect(retryClient.getMe).toHaveBeenCalledTimes(2)
    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('renders the Clear page route', async () => {
    const { wrapper } = await mountAt('/rooms/room-1/clear')

    expect(wrapper.get('h1').text()).toBe('Clear')
  })

  it('renders the development device PoC page without changing App composition', async () => {
    const { wrapper } = await mountAt('/device-poc')

    expect(wrapper.get('h1').text()).toBe('Device Web Serial raw viewer')
  })
})

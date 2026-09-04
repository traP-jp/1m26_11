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
import problemResponse from '../../../openapi/examples/problems/available-response.json'
import roomActive from '../../../openapi/examples/rooms/response-active.json'
import currentRun from '../../../openapi/examples/runs/active-response.json'
import newRun from '../../../openapi/examples/runs/start-new-response.json'
import answerCleared from '../../../openapi/examples/answers/response-correct-cleared.json'
import answerIncorrect from '../../../openapi/examples/answers/response-incorrect.json'
import queryIncorrect from '../../../openapi/examples/queries/response-incorrect.json'
import {
  ApiClientError,
  type ApiClient,
  type GetCurrentRunResponse,
  type GetMeResponse,
  type GetProblemResponse,
  type GetRoomResponse,
  type LoginGuestResponse,
  type StartOrResumeRunResponse,
  type SubmitAnswerResponse,
  type SubmitQueryResponse,
} from '../api/client'
import {
  authApiClientKey,
  authNavigationHandlerKey,
  type AuthNavigationHandler,
} from '../utils/auth'

const stringProblemResponse: GetProblemResponse = {
  ...(problemResponse as GetProblemResponse),
  submission_type: 'string',
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((next) => {
    resolve = next
  })
  return { promise, resolve }
}

function createAppApiClient(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(meAuthenticated as GetMeResponse),
    loginGuest: vi.fn<ApiClient['loginGuest']>(),
    logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    getRoom: vi.fn<ApiClient['getRoom']>(({ room_id }) =>
      Promise.resolve({ ...(roomActive as GetRoomResponse), id: room_id }),
    ),
    startOrResumeRun: vi
      .fn<ApiClient['startOrResumeRun']>()
      .mockResolvedValue(newRun as StartOrResumeRunResponse),
    getCurrentRun: vi.fn<ApiClient['getCurrentRun']>().mockRejectedValue(
      new ApiClientError('挑戦中のrunが見つかりません', {
        kind: 'http',
        status: 404,
        code: 'RUN_NOT_FOUND',
        details: {},
      }),
    ),
    getProblem: vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse),
    submitQuery: vi
      .fn<ApiClient['submitQuery']>()
      .mockResolvedValue(queryIncorrect as SubmitQueryResponse),
    submitAnswer: vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerIncorrect as SubmitAnswerResponse),
    ...overrides,
  }
}

async function mountAt(
  path: string,
  client: ApiClient = createAppApiClient(),
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
    const client = createAppApiClient()
    const { router, wrapper } = await mountAt('/', client)

    wrapper.getComponent(PortalPage).vm.$emit('startRoom', '1411824c-d357-4941-af76-c76cb827dda6')
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/rooms/1411824c-d357-4941-af76-c76cb827dda6')
    expect(wrapper.findComponent(RoomPage).exists()).toBe(true)
    expect(client.getRoom).toHaveBeenCalledExactlyOnceWith({
      room_id: '1411824c-d357-4941-af76-c76cb827dda6',
    })
    expect(client.getCurrentRun).not.toHaveBeenCalled()
    expect(client.startOrResumeRun).toHaveBeenCalledExactlyOnceWith({
      room_id: '1411824c-d357-4941-af76-c76cb827dda6',
    })
    expect(client.getProblem).toHaveBeenCalledExactlyOnceWith({
      room_id: '1411824c-d357-4941-af76-c76cb827dda6',
      problem_id: problemResponse.id,
    })

    wrapper.getComponent(RoomPage).vm.$emit('uiEvent', { type: 'room-exited' })
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/')
    expect(wrapper.findComponent(PortalPage).exists()).toBe(true)
  })

<<<<<<< HEAD
  it('restores the current run without starting another one on a direct Room visit', async () => {
    const getCurrentRun = vi
      .fn<ApiClient['getCurrentRun']>()
      .mockResolvedValue(currentRun as GetCurrentRunResponse)
    const startOrResumeRun = vi.fn<ApiClient['startOrResumeRun']>()
    const client = createAppApiClient({ getCurrentRun, startOrResumeRun })

    const { wrapper } = await mountAt('/rooms/1411824c-d357-4941-af76-c76cb827dda6', client)

    expect(getCurrentRun).toHaveBeenCalledExactlyOnceWith({
      room_id: '1411824c-d357-4941-af76-c76cb827dda6',
    })
    expect(startOrResumeRun).not.toHaveBeenCalled()
    expect(wrapper.getComponent(RoomPage).props('viewModel')).toMatchObject({
      room: {
        id: '1411824c-d357-4941-af76-c76cb827dda6',
        number: roomActive.number,
        name: roomActive.name,
      },
      serverElapsedMs: currentRun.elapsed_ms,
      problems: [{ id: problemResponse.id, status: 'cleared', selected: true }],
      clear: { clearedCount: currentRun.cleared_problem_ids.length },
    })
  })

  it('keeps the newest Room when an older restore finishes later', async () => {
    const firstRoomId = '1411824c-d357-4941-af76-c76cb827dda6'
    const secondRoomId = '1411824c-d357-4941-af76-c76cb827dda7'
    const firstRun = deferred<GetCurrentRunResponse>()
    const getCurrentRun = vi.fn<ApiClient['getCurrentRun']>(({ room_id }) =>
      room_id === firstRoomId
        ? firstRun.promise
        : Promise.resolve(currentRun as GetCurrentRunResponse),
    )
    const getProblem = vi
      .fn<ApiClient['getProblem']>()
      .mockResolvedValue(problemResponse as GetProblemResponse)
    const client = createAppApiClient({ getCurrentRun, getProblem })
    const { router, wrapper } = await mountAt(`/rooms/${firstRoomId}`, client)

    await router.push(`/rooms/${secondRoomId}`)
    await flushPromises()

    expect(wrapper.getComponent(RoomPage).props('viewModel').room.id).toBe(secondRoomId)

    firstRun.resolve(currentRun as GetCurrentRunResponse)
    await flushPromises()

    expect(wrapper.getComponent(RoomPage).props('viewModel').room.id).toBe(secondRoomId)
    expect(getProblem).toHaveBeenCalledExactlyOnceWith({
      room_id: secondRoomId,
      problem_id: problemResponse.id,
    })
  })

  it('connects operation input to the shared buffer and query controller', async () => {
    const submitQuery = vi
      .fn<ApiClient['submitQuery']>()
      .mockResolvedValue(queryIncorrect as SubmitQueryResponse)
    const client = createAppApiClient({ submitQuery })
    const { wrapper } = await mountAt('/rooms/1411824c-d357-4941-af76-c76cb827dda6', client)
    const roomPage = wrapper.getComponent(RoomPage)

    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'down',
      count: 1,
    })
    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'down',
      count: 1,
    })
    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'right',
      count: 1,
    })
    await wrapper.vm.$nextTick()

    expect(roomPage.props('viewModel').queryInput.operations).toEqual([
      { control: 'down', count: 2 },
      { control: 'right', count: 1 },
    ])

    roomPage.vm.$emit('uiEvent', { type: 'query-operation-removed', index: 0 })
    await wrapper.vm.$nextTick()
    expect(roomPage.props('viewModel').queryInput.operations).toEqual([
      { control: 'right', count: 1 },
    ])

    roomPage.vm.$emit('uiEvent', { type: 'query-operations-cleared' })
    await wrapper.vm.$nextTick()
    expect(roomPage.props('viewModel').queryInput.operations).toEqual([])

    roomPage.vm.$emit('uiEvent', { type: 'query-submitted', source: 'keyboard' })
    await flushPromises()
    expect(submitQuery).not.toHaveBeenCalled()

    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'right',
      count: problemResponse.input_schema.query.max_operations,
    })
    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'right',
      count: 1,
    })
    await wrapper.vm.$nextTick()
    expect(roomPage.props('viewModel').queryInput.operations).toEqual([
      { control: 'right', count: problemResponse.input_schema.query.max_operations },
    ])

    roomPage.vm.$emit('uiEvent', { type: 'query-operations-cleared' })
    roomPage.vm.$emit('uiEvent', {
      type: 'condition-changed',
      source: 'keyboard',
      control: 'right',
      count: 1,
    })
    roomPage.vm.$emit('uiEvent', { type: 'query-submitted', source: 'keyboard' })
    roomPage.vm.$emit('uiEvent', { type: 'query-submitted', source: 'keyboard' })
    await flushPromises()

    expect(submitQuery).toHaveBeenCalledExactlyOnceWith(
      {
        room_id: '1411824c-d357-4941-af76-c76cb827dda6',
        problem_id: problemResponse.id,
      },
      { source: 'keyboard', operations: [{ control: 'right', count: 1 }] },
    )
    expect(roomPage.props('viewModel').queryInput.operations).toEqual([])
    expect(roomPage.props('viewModel').answerJudgement.state).toBe('incorrect')
  })

  it('connects string answers to the answer controller and inline ClearScreen', async () => {
    const submitAnswer = vi
      .fn<ApiClient['submitAnswer']>()
      .mockResolvedValue(answerCleared as SubmitAnswerResponse)
    const submitQuery = vi.fn<ApiClient['submitQuery']>()
    const getProblem = vi.fn<ApiClient['getProblem']>().mockResolvedValue(stringProblemResponse)
    const client = createAppApiClient({ getProblem, submitQuery, submitAnswer })
    const { wrapper } = await mountAt('/rooms/1411824c-d357-4941-af76-c76cb827dda6', client)
    const roomPage = wrapper.getComponent(RoomPage)

    roomPage.vm.$emit('uiEvent', { type: 'query-submitted', source: 'keyboard' })
    roomPage.vm.$emit('uiEvent', {
      type: 'answer-submitted',
      source: 'mouse',
      answer: '19520715',
    })
    await flushPromises()

    expect(submitQuery).not.toHaveBeenCalled()
    expect(submitAnswer).toHaveBeenCalledExactlyOnceWith(
      {
        room_id: '1411824c-d357-4941-af76-c76cb827dda6',
        problem_id: stringProblemResponse.id,
      },
      { answer: '19520715' },
    )
    expect(wrapper.getComponent(RoomPage).props('viewModel')).toMatchObject({
      serverElapsedMs: answerCleared.elapsed_ms,
      clear: {
        cleared: true,
        clearedCount: answerCleared.progress.cleared_count,
        requiredCount: answerCleared.progress.required_count,
      },
    })
=======
  it('renders the development problem authoring page without changing App composition', async () => {
    const roomId = '1411824c-d357-4941-af76-c76cb827dda6'
    const { router, wrapper } = await mountAt(`/author/rooms/${roomId}/problems/new`)

    expect(router.currentRoute.value.name).toBe('problem-author-new')
    expect(wrapper.getComponent(AuthorProblemPage).props('roomId')).toBe(roomId)
>>>>>>> main
  })

  it('connects Demo guest login to the auth flow', async () => {
    const guestClient = createAppApiClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValueOnce(meUnauthenticated as GetMeResponse)
        .mockResolvedValueOnce(meAuthenticated as GetMeResponse),
      loginGuest: vi
        .fn<ApiClient['loginGuest']>()
        .mockResolvedValue(guestResponse as LoginGuestResponse),
    })
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
    const neoClient = createAppApiClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValue(meNeoshowcaseUnauthenticated as GetMeResponse),
      loginGuest: vi.fn<ApiClient['loginGuest']>(),
      logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    })
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
    const neoClient = createAppApiClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockResolvedValue(meNeoshowcaseAuthenticated as GetMeResponse),
      loginGuest: vi.fn<ApiClient['loginGuest']>(),
      logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    })
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
    const retryClient = createAppApiClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockRejectedValueOnce(new Error('me failed'))
        .mockResolvedValueOnce(meAuthenticated as GetMeResponse),
    })
    const { wrapper } = await mountAt('/', retryClient)

    expect(wrapper.get('[role="alert"]').text()).toContain('認証状態を取得できませんでした。')
    await wrapper.get('[role="alert"] button').trigger('click')
    await flushPromises()

    expect(retryClient.getMe).toHaveBeenCalledTimes(2)
    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('retries authentication from a direct Room visit before loading the Room', async () => {
    const retryClient = createAppApiClient({
      getMe: vi
        .fn<ApiClient['getMe']>()
        .mockRejectedValueOnce(new Error('me failed'))
        .mockResolvedValueOnce(meAuthenticated as GetMeResponse),
    })
    const { wrapper } = await mountAt('/rooms/1411824c-d357-4941-af76-c76cb827dda6', retryClient)

    expect(wrapper.get('[role="alert"]').text()).toContain('認証状態を取得できませんでした。')
    await wrapper.get('[role="alert"] button').trigger('click')
    await flushPromises()

    expect(retryClient.getMe).toHaveBeenCalledTimes(2)
    expect(wrapper.findComponent(RoomPage).exists()).toBe(true)
  })

  it('redirects the legacy Clear route to the canonical RoomPage route', async () => {
    const roomId = '1411824c-d357-4941-af76-c76cb827dda6'
    const { router, wrapper } = await mountAt(`/rooms/${roomId}/clear`)

    expect(router.currentRoute.value).toMatchObject({ name: 'room', params: { roomId } })
    expect(wrapper.findComponent(RoomPage).exists()).toBe(true)
  })

  it('renders the development device PoC page without changing App composition', async () => {
    const { wrapper } = await mountAt('/device-poc')

    expect(wrapper.get('h1').text()).toBe('Device Web Serial raw viewer')
  })
})

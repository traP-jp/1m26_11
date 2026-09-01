import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, type Router } from 'vue-router'

import App from '../App.vue'
import PortalPage from '../PortalPage.vue'
import RoomPage from '../RoomPage.vue'
import { createAppRouter } from '../router'

async function mountAt(
  path: string,
): Promise<{ router: Router; wrapper: ReturnType<typeof mount> }> {
  const router = createAppRouter(createMemoryHistory())
  await router.push(path)
  const wrapper = mount(App, { global: { plugins: [router] } })
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

  it('renders the Clear page route', async () => {
    const { wrapper } = await mountAt('/rooms/room-1/clear')

    expect(wrapper.get('h1').text()).toBe('Clear')
  })

  it('renders the development device PoC page without changing App composition', async () => {
    const { wrapper } = await mountAt('/device-poc')

    expect(wrapper.get('h1').text()).toBe('Device Web Serial raw viewer')
  })
})

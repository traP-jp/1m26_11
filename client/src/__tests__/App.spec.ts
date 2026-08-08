import { describe, it, expect } from 'vitest'

import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import App from '../App.vue'

describe('App', () => {
  it('renders the project setup', () => {
    const wrapper = mount(App)

    expect(wrapper.get('h1').text()).toBe('Frontend is ready.')
    expect(wrapper.get('button').text()).toBe('Open actions')
    expect(wrapper.get('a').attributes('href')).toBe('/openapi.yaml')
    expect(wrapper.findAll('.stack li')).toHaveLength(4)
  })

  it('opens the headless ui menu', async () => {
    const wrapper = mount(App)

    await wrapper.get('button').trigger('click')
    await nextTick()

    expect(wrapper.text()).toContain('Save changes')
    expect(wrapper.text()).toContain('Duplicate')
    expect(wrapper.text()).toContain('Archive')
  })
})

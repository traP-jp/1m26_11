import { describe, it, expect } from 'vitest'

import { mount } from '@vue/test-utils'
import App from '../App.vue'

describe('App', () => {
  it('renders the project setup', () => {
    const wrapper = mount(App)

    expect(wrapper.get('h1').text()).toBe('Frontend is ready.')
    expect(wrapper.get('a').attributes('href')).toBe('/openapi.yaml')
    expect(wrapper.findAll('.stack li')).toHaveLength(4)
  })
})

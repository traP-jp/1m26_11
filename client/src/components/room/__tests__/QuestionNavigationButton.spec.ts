import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import QuestionNavigationButton from '../QuestionNavigationButton.vue'
import { questionNavigationButtonFixture } from '../QuestionNavigationButton.fixture'

describe('QuestionNavigationButton', () => {
  it.each([questionNavigationButtonFixture.previous, questionNavigationButtonFixture.next])(
    'renders the $direction variant and emits the supplied problem ID once',
    async (fixture) => {
      const wrapper = mount(QuestionNavigationButton, {
        props: { ...fixture, disabled: false },
      })

      await wrapper.get('button').trigger('click')

      expect(wrapper.emitted('select')).toEqual([[fixture.problemId]])
    },
  )

  it('does not emit while disabled', async () => {
    const wrapper = mount(QuestionNavigationButton, {
      props: { ...questionNavigationButtonFixture.next, disabled: true },
    })

    await wrapper.get('button').trigger('click')

    expect(wrapper.get('button').attributes('disabled')).toBeDefined()
    expect(wrapper.emitted('select')).toBeUndefined()
  })
})

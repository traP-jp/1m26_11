import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import AnswerPanel from '../AnswerPanel.vue'
import { answerPanelFixture } from '../AnswerPanel.fixture'

const defaultProps = {
  maxLength: answerPanelFixture.maxLength,
  pending: false,
  disabled: false,
}

describe('AnswerPanel', () => {
  it('always renders an accessible input with the fixture maximum length', () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })
    const textarea = wrapper.get('textarea[name="answer"]')
    const label = wrapper.get('label')

    expect(wrapper.get('section').attributes('id')).toBe('answer-panel')
    expect(label.attributes('for')).toBe(textarea.attributes('id'))
    expect(textarea.attributes('maxlength')).toBe(String(answerPanelFixture.maxLength))
    expect(wrapper.text()).toContain(`最大${answerPanelFixture.maxLength}文字`)
  })

  it('emits the unchanged fixture answer exactly once through form submission', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })

    await wrapper.get('textarea').setValue(answerPanelFixture.submittedAnswer)
    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([[answerPanelFixture.submittedAnswer, 'mouse']])
  })

  it('emits the answer exactly once when the submit button is clicked', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps, attachTo: document.body })
    const submitButton = wrapper.get<HTMLButtonElement>('button[type="submit"]')

    await wrapper.get('textarea').setValue(answerPanelFixture.submittedAnswer)
    submitButton.element.click()
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('submit')).toEqual([[answerPanelFixture.submittedAnswer, 'mouse']])

    wrapper.unmount()
  })

  it('preserves whitespace and allows an empty answer defined by the transport schema', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })
    const textarea = wrapper.get('textarea')

    await textarea.setValue('  answer\n')
    await wrapper.get('form').trigger('submit')
    await textarea.setValue('')
    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([
      ['  answer\n', 'mouse'],
      ['', 'mouse'],
    ])
  })

  it('submits once on Enter and ignores repeated Enter presses', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })
    const textarea = wrapper.get('textarea')

    await textarea.setValue(answerPanelFixture.submittedAnswer)
    const enter = new KeyboardEvent('keydown', {
      key: 'Enter',
      bubbles: true,
      cancelable: true,
    })
    const repeatedEnter = new KeyboardEvent('keydown', {
      key: 'Enter',
      repeat: true,
      bubbles: true,
      cancelable: true,
    })
    textarea.element.dispatchEvent(enter)
    textarea.element.dispatchEvent(repeatedEnter)

    expect(wrapper.emitted('submit')).toEqual([[answerPanelFixture.submittedAnswer, 'keyboard']])
    expect(enter.defaultPrevented).toBe(true)
    expect(repeatedEnter.defaultPrevented).toBe(true)
  })

  it('does not submit with modifiers or while an IME composition is active', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })
    const textarea = wrapper.get('textarea')

    await textarea.setValue(answerPanelFixture.submittedAnswer)
    const modifiedEnters = [
      { altKey: true },
      { ctrlKey: true },
      { metaKey: true },
      { shiftKey: true },
    ].map(
      (modifier) =>
        new KeyboardEvent('keydown', {
          key: 'Enter',
          ...modifier,
          bubbles: true,
          cancelable: true,
        }),
    )
    const composingEnter = new KeyboardEvent('keydown', {
      key: 'Enter',
      isComposing: true,
      bubbles: true,
      cancelable: true,
    })
    for (const event of modifiedEnters) textarea.element.dispatchEvent(event)
    textarea.element.dispatchEvent(composingEnter)

    expect(wrapper.emitted('submit')).toBeUndefined()
    expect(modifiedEnters.every((event) => !event.defaultPrevented)).toBe(true)
    expect(composingEnter.defaultPrevented).toBe(false)
  })

  it.each([
    { pending: true, disabled: false },
    { pending: false, disabled: true },
  ])('blocks input and submission while unavailable: %o', async (props) => {
    const wrapper = mount(AnswerPanel, {
      props: { ...defaultProps, ...props },
    })
    const textarea = wrapper.get('textarea')

    expect(textarea.attributes('disabled')).toBeDefined()
    expect(wrapper.get('button[type="submit"]').attributes('disabled')).toBeDefined()

    await wrapper.get('form').trigger('submit')
    await textarea.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('submit')).toBeUndefined()
  })

  it('keeps the draft while pending changes', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })

    await wrapper.get('textarea').setValue(answerPanelFixture.submittedAnswer)
    await wrapper.setProps({ pending: true })
    expect(wrapper.get('textarea').element.value).toBe(answerPanelFixture.submittedAnswer)

    await wrapper.setProps({ pending: false })

    expect(wrapper.get('textarea').element.value).toBe(answerPanelFixture.submittedAnswer)
  })

  it('accepts the exact maximum and rejects a programmatic value over it', async () => {
    const wrapper = mount(AnswerPanel, { props: defaultProps })
    const textarea = wrapper.get('textarea')

    const maximumAnswer = 'x'.repeat(answerPanelFixture.maxLength)
    await textarea.setValue(maximumAnswer)
    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([[maximumAnswer, 'mouse']])

    await textarea.setValue('x'.repeat(answerPanelFixture.maxLength + 1))
    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([[maximumAnswer, 'mouse']])
    expect(wrapper.get('button[type="submit"]').attributes('disabled')).toBeDefined()
  })
})

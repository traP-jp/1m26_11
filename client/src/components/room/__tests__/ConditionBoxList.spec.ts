import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import ConditionBoxList from '../ConditionBoxList.vue'
import { conditionBoxListFixture, firstConditionBoxItem } from '../ConditionBoxList.fixture'

const defaultProps = {
  items: conditionBoxListFixture.items,
  disabled: false,
}

describe('ConditionBoxList', () => {
  it('renders supplied items in order', () => {
    const wrapper = mount(ConditionBoxList, { props: defaultProps })

    expect(wrapper.findAll('li').map((item) => item.find('span').text())).toEqual([
      '上を2回',
      '右を1回',
    ])
  })

  it('emits add, remove with the supplied ID, and clear exactly once', async () => {
    const wrapper = mount(ConditionBoxList, { props: defaultProps })

    await wrapper.get('button:nth-of-type(1)').trigger('click')
    await wrapper.get(`button[aria-label="${firstConditionBoxItem.label}を削除"]`).trigger('click')
    await wrapper.get('button:nth-of-type(2)').trigger('click')

    expect(wrapper.emitted('add')).toEqual([[]])
    expect(wrapper.emitted('remove')).toEqual([[firstConditionBoxItem.id]])
    expect(wrapper.emitted('clear')).toEqual([[]])
  })

  it('does not emit any operation while disabled', async () => {
    const wrapper = mount(ConditionBoxList, {
      props: { ...defaultProps, disabled: true },
    })

    for (const button of wrapper.findAll('button')) await button.trigger('click')

    expect(
      wrapper.findAll('button').every((button) => button.attributes('disabled') !== undefined),
    ).toBe(true)
    expect(wrapper.emitted('add')).toBeUndefined()
    expect(wrapper.emitted('remove')).toBeUndefined()
    expect(wrapper.emitted('clear')).toBeUndefined()
  })
})

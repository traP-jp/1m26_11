import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { SerialStatus } from '../SerialStatusNotice.types'
import SerialStatusNotice from '../SerialStatusNotice.vue'

const presentations = [
  ['unsupported', 'この環境ではSerialを利用できません', 'status'],
  ['denied', 'Serial接続が許可されませんでした', 'alert'],
  ['connecting', 'Serialへ接続中です', 'status'],
  ['connected', 'Serialに接続しました', 'status'],
  ['disconnected', 'Serialは接続されていません', 'status'],
  ['retry-failed', 'Serialの再接続に失敗しました', 'alert'],
] as const satisfies ReadonlyArray<readonly [SerialStatus, string, 'alert' | 'status']>

const retryStatuses = [
  'denied',
  'disconnected',
  'retry-failed',
] as const satisfies readonly SerialStatus[]
const noRetryStatuses = [
  'unsupported',
  'connecting',
  'connected',
] as const satisfies readonly SerialStatus[]

describe('SerialStatusNotice', () => {
  it.each(presentations)('%s状態を同じbanner shellで区別して表示する', (status, title, role) => {
    const wrapper = mount(SerialStatusNotice, { props: { status } })
    const notice = wrapper.get('[data-testid="serial-status-notice"]')

    expect(notice.attributes('data-status')).toBe(status)
    expect(notice.attributes('role')).toBe(role)
    expect(notice.attributes('aria-live')).toBe(role === 'alert' ? 'assertive' : 'polite')
    expect(notice.classes()).toEqual(
      expect.arrayContaining(['flex', 'w-full', 'rounded-xl', 'border', 'px-4', 'py-3']),
    )
    expect(wrapper.get('[data-testid="serial-status-title"]').text()).toBe(title)
    expect(wrapper.findAll('p')[1]?.text()).not.toBe('')
    expect(wrapper.get('[aria-label="入力方法を選択"]').attributes('role')).toBe('group')
    expect(wrapper.emitted()).toEqual({})
  })

  it('接続中だけbusyとして通知し、代替入力操作を無効化する', async () => {
    const wrapper = mount(SerialStatusNotice, { props: { status: 'connecting' } })
    const keyboard = wrapper.get<HTMLButtonElement>('[data-testid="use-keyboard"]')
    const screen = wrapper.get<HTMLButtonElement>('[data-testid="use-screen"]')

    expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('aria-busy')).toBe('true')
    expect(keyboard.element.disabled).toBe(true)
    expect(screen.element.disabled).toBe(true)

    keyboard.element.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    screen.element.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('use-keyboard')).toBeUndefined()
    expect(wrapper.emitted('use-screen')).toBeUndefined()

    await wrapper.setProps({ status: 'connected' })

    expect(
      wrapper.get('[data-testid="serial-status-notice"]').attributes('aria-busy'),
    ).toBeUndefined()
    expect(keyboard.element.disabled).toBe(false)
    expect(screen.element.disabled).toBe(false)
  })

  it.each(retryStatuses)('%s状態だけ再接続操作をemitする', async (status) => {
    const wrapper = mount(SerialStatusNotice, { props: { status } })

    await wrapper.get('[data-testid="serial-retry"]').trigger('click')

    expect(wrapper.emitted('retry')).toEqual([[]])
  })

  it('復旧操作のlabelとdisabled状態を呼出し側から明示できる', async () => {
    const wrapper = mount(SerialStatusNotice, {
      props: {
        status: 'disconnected',
        retryLabel: 'Serialを解放する',
        retryDisabled: true,
      },
    })
    const retry = wrapper.get<HTMLButtonElement>('[data-testid="serial-retry"]')

    expect(retry.text()).toBe('Serialを解放する')
    expect(retry.element.disabled).toBe(true)
    await retry.trigger('click')
    expect(wrapper.emitted('retry')).toBeUndefined()
  })

  it.each(noRetryStatuses)('%s状態では再接続操作を表示しない', (status) => {
    const wrapper = mount(SerialStatusNotice, { props: { status } })

    expect(wrapper.find('[data-testid="serial-retry"]').exists()).toBe(false)
  })

  it('非対応時にbrowser対応とsecure contextの両方を案内する', () => {
    const wrapper = mount(SerialStatusNotice, { props: { status: 'unsupported' } })

    expect(wrapper.text()).toContain('Web Serial対応ブラウザ')
    expect(wrapper.text()).toContain('HTTPSまたはlocalhost')
  })

  it('キーボードと画面ボタンへの切替を別々のeventとしてemitする', async () => {
    const wrapper = mount(SerialStatusNotice, { props: { status: 'disconnected' } })

    await wrapper.get('[data-testid="use-keyboard"]').trigger('click')
    await wrapper.get('[data-testid="use-screen"]').trigger('click')

    expect(wrapper.emitted('use-keyboard')).toEqual([[]])
    expect(wrapper.emitted('use-screen')).toEqual([[]])
  })
})

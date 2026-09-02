import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import type { SerialConnectionState } from '../../../input/useWebSerialConnection'
import SerialConnectControl from '../SerialConnectControl.vue'

function mountControl(
  state: SerialConnectionState,
  options: Partial<{
    busy: boolean
    canConnect: boolean
    canRetry: boolean
    canDisconnect: boolean
  }> = {},
) {
  return mount(SerialConnectControl, {
    props: {
      state,
      busy: false,
      canConnect: false,
      canRetry: false,
      canDisconnect: false,
      ...options,
    },
  })
}

describe('SerialConnectControl', () => {
  it.each<[SerialConnectionState, string]>([
    [
      {
        phase: 'unsupported',
        reason: 'api-unavailable',
        message: 'Web Serial APIに対応していません。',
      },
      'Web Serial非対応',
    ],
    [{ phase: 'idle', message: '未接続です。' }, 'Serial未接続'],
    [{ phase: 'requesting', attempt: 'connect', message: '選択してください。' }, 'Serial接続中'],
    [{ phase: 'connected', message: '読取り中です。' }, 'Serial接続済み'],
    [
      { phase: 'disconnected', reason: 'device-disconnected', message: '切断されました。' },
      'Serial切断',
    ],
    [{ phase: 'error', operation: 'read', message: '読取りに失敗しました。' }, 'Serial接続エラー'],
  ])('$phase状態を表示する', (state, title) => {
    const wrapper = mountControl(state)

    expect(wrapper.text()).toContain(title)
    expect(wrapper.text()).toContain(state.message)
    expect(wrapper.emitted()).toEqual({})
  })

  it('connect、disconnect、retryを利用者のclick時だけemitする', async () => {
    const idle = mountControl({ phase: 'idle', message: '未接続です。' }, { canConnect: true })
    await idle.get('button').trigger('click')
    expect(idle.emitted('connect')).toEqual([[]])

    const connected = mountControl(
      { phase: 'connected', message: '読取り中です。' },
      { canDisconnect: true },
    )
    const disconnect = connected
      .findAll('button')
      .find((button) => button.text() === 'Serialを切断')
    expect(disconnect).toBeDefined()
    await disconnect!.trigger('click')
    expect(connected.emitted('disconnect')).toEqual([[]])

    const disconnected = mountControl(
      { phase: 'disconnected', reason: 'user', message: '切断しました。' },
      { canRetry: true },
    )
    const retry = disconnected.findAll('button').find((button) => button.text() === '再接続')
    expect(retry).toBeDefined()
    await retry!.trigger('click')
    expect(disconnected.emitted('retry')).toEqual([[]])
  })

  it('代替入力への切替を明示的にemitする', async () => {
    const wrapper = mountControl({
      phase: 'error',
      operation: 'reconnect',
      message: '再接続できませんでした。',
    })
    const alternative = wrapper
      .findAll('button')
      .find((button) => button.text() === 'キーボード／画面ボタンで続ける')

    expect(alternative).toBeDefined()
    await alternative!.trigger('click')
    expect(wrapper.emitted('switchAlternative')).toEqual([[]])
  })

  it('errorをalertとして通知し、close失敗時はport解放の再試行を提供する', async () => {
    const wrapper = mountControl(
      {
        phase: 'error',
        operation: 'close-port',
        message: 'portを解放できませんでした。',
      },
      { canDisconnect: true },
    )

    expect(wrapper.get('[role="alert"]').text()).toContain('portを解放できませんでした。')
    const retryClose = wrapper
      .findAll('button')
      .find((button) => button.text() === 'ポート解放を再試行')
    expect(retryClose).toBeDefined()
    await retryClose!.trigger('click')
    expect(wrapper.emitted('disconnect')).toEqual([[]])
  })
})

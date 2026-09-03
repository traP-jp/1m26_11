import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import SerialRawViewer from '../SerialRawViewer.vue'
import { serialRawViewerFixtures } from '../SerialRawViewer.fixture'

describe('SerialRawViewer', () => {
  it('unsupported理由をalert以外のstatusとして表示しConnectを無効化する', () => {
    const wrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.unsupported,
    })

    expect(wrapper.get('[data-testid="serial-status"]').attributes('role')).toBe('status')
    expect(wrapper.get('[data-testid="serial-status"]').text()).toContain('対応していません')
    expect(wrapper.get('button').attributes('disabled')).toBeDefined()
  })

  it('raw byteをhexで表示し、chunkをframeと扱わない注意を表示する', () => {
    const wrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.running,
    })

    expect(wrapper.get('[data-testid="raw-hex"]').text()).toMatch(/^7b 22 76/)
    expect(wrapper.text()).toContain('chunk境界はframe境界ではありません')
    expect(wrapper.get('[data-testid="decoded-preview"]').text()).toContain('␍␊')
    expect(wrapper.get('[data-testid="decoded-preview"]').text()).toContain(
      '"gesture":"short_press"',
    )
  })

  it('各操作をsemantic eventとして通知する', async () => {
    const runningWrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.running,
    })
    const runningButtons = runningWrapper.findAll('button')

    await runningButtons[1]!.trigger('click')

    expect(runningWrapper.emitted('stop')).toHaveLength(1)

    const disconnectedWrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.disconnected,
    })
    const disconnectedButtons = disconnectedWrapper.findAll('button')

    await disconnectedButtons[2]!.trigger('click')
    await disconnectedButtons[3]!.trigger('click')

    expect(disconnectedWrapper.emitted('downloadRaw')).toHaveLength(1)
    expect(disconnectedWrapper.emitted('downloadMetadata')).toHaveLength(1)
  })

  it('errorをalertとして通知する', () => {
    const wrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.error,
    })

    expect(wrapper.get('[data-testid="serial-status"]').attributes('role')).toBe('alert')
  })
})

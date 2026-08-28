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

    expect(wrapper.get('[data-testid="raw-hex"]').text()).toMatch(/^5b 62 75 74/)
    expect(wrapper.text()).toContain('chunk境界はframe境界ではありません')
    expect(wrapper.get('[data-testid="decoded-preview"]').text()).toContain('␍␊')
  })

  it('各操作をsemantic eventとして通知する', async () => {
    const wrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.running,
    })
    const buttons = wrapper.findAll('button')

    await buttons[1]!.trigger('click')
    await buttons[2]!.trigger('click')
    await buttons[3]!.trigger('click')

    expect(wrapper.emitted('stop')).toHaveLength(1)
    expect(wrapper.emitted('downloadRaw')).toHaveLength(1)
    expect(wrapper.emitted('downloadMetadata')).toHaveLength(1)
  })

  it('errorをalertとして通知する', () => {
    const wrapper = mount(SerialRawViewer, {
      props: serialRawViewerFixtures.error,
    })

    expect(wrapper.get('[data-testid="serial-status"]').attributes('role')).toBe('alert')
  })
})

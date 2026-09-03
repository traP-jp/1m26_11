import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import ProblemBodyAssets from '../ProblemBodyAssets.vue'
import { problemBodyAssetsFixture } from '../ProblemBodyAssets.fixture'
import QuestionArea from '../QuestionArea.vue'
import QuestionHeader from '../QuestionHeader.vue'

describe('QuestionArea', () => {
  it('composes the header and body with the supplied public props', () => {
    const props = {
      problemNumber: 2,
      title: '暗号の書かれた地図',
      problemType: 'small' as const,
      smallIndex: 2,
      smallTotal: 3,
      ...problemBodyAssetsFixture,
    }
    const wrapper = mount(QuestionArea, { props })

    expect(wrapper.findComponent(QuestionHeader).props()).toMatchObject({
      problemNumber: props.problemNumber,
      title: props.title,
      problemType: props.problemType,
      smallIndex: props.smallIndex,
      smallTotal: props.smallTotal,
    })
    expect(wrapper.findComponent(ProblemBodyAssets).props()).toMatchObject({
      bodyMarkdown: props.bodyMarkdown,
      assets: props.assets,
    })
  })
})

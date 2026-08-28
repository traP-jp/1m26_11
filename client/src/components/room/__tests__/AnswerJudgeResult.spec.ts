import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import AnswerJudgeResult from '../AnswerJudgeResult.vue'
import { answerJudgeResultFixtures } from '../AnswerJudgeResult.fixture'

describe('AnswerJudgeResult', () => {
  it.each([
    ['idle', '判定待ち', '—', 'bg-[#eef3fa]'],
    ['pending', '判定中', '…', 'bg-[#e7f0ff]'],
    ['correct', '正解', '○', 'bg-[#ddf8e8]'],
    ['incorrect', '不正解', '×', 'bg-[#ffebec]'],
    ['error', '判定エラー', '!', 'bg-[#fff1da]'],
  ] as const)('%sの文言、記号、配色を表示する', (state, label, symbol, colorClass) => {
    const wrapper = mount(AnswerJudgeResult, {
      props: { state },
    })

    expect(wrapper.attributes('data-state')).toBe(state)
    expect(wrapper.get('[data-testid="result-label"]').text()).toBe(label)
    expect(wrapper.get('[data-testid="result-symbol"]').text()).toBe(symbol)
    expect(wrapper.get('[data-testid="result-badge"]').classes()).toContain(colorClass)
    expect(wrapper.attributes('aria-live')).toBe('polite')
    expect(wrapper.emitted()).toEqual({})
  })

  it('pendingの間だけbusyであることを通知する', async () => {
    const wrapper = mount(AnswerJudgeResult, {
      props: { state: 'idle' },
    })

    expect(wrapper.attributes('aria-busy')).toBeUndefined()

    await wrapper.setProps({ state: 'pending' })

    expect(wrapper.attributes('aria-busy')).toBe('true')

    await wrapper.setProps({ state: 'correct' })

    expect(wrapper.attributes('aria-busy')).toBeUndefined()
  })

  it('errorとincorrectを文言で区別して通知する', () => {
    const error = mount(AnswerJudgeResult, {
      props: { state: 'error' },
    })
    const incorrect = mount(AnswerJudgeResult, {
      props: { state: 'incorrect' },
    })

    expect(error.attributes('aria-live')).toBe('polite')
    expect(error.text()).toContain('判定結果を取得できませんでした')
    expect(incorrect.attributes('aria-live')).toBe('polite')
    expect(incorrect.text()).toContain('回答は不正解です')
  })

  it('共有answer fixtureをcorrectとincorrectの判定stateへ写像する', async () => {
    const wrapper = mount(AnswerJudgeResult, {
      props: answerJudgeResultFixtures.incorrect,
    })

    expect(wrapper.attributes('data-state')).toBe('incorrect')

    await wrapper.setProps({ ...answerJudgeResultFixtures.correct })

    expect(wrapper.attributes('data-state')).toBe('correct')
  })
})

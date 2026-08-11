import { describe, expect, it } from 'vitest'

import { mockContract } from '../data'
import { createMockState } from '../state'

describe('mock backend state', () => {
  it('starts with frontend-friendly defaults', () => {
    const state = createMockState(mockContract)

    expect(state.snapshot()).toMatchObject({
      auth_mode: 'demo',
      authenticated: true,
      active_run_exists: false,
      problem_status: 'available',
    })
  })

  it('starts from a selected scenario and applies its state transitions', () => {
    const state = createMockState(mockContract, 'answer_correct_and_clear_run')

    expect(state.get('last_required_problem')).toBe(true)
    const step = state.applyStep('answer_correct_and_clear_run', 'submitAnswer')

    expect(step.response.example).toBe('correct_answer_clears_run')
    expect(state.snapshot()).toMatchObject({
      problem_status: 'cleared',
      run_status: 'cleared',
    })
  })

  it('supports the login and logout sequence from one scenario', () => {
    const state = createMockState(mockContract, 'demo_login_and_logout')

    state.applyStep('demo_login_and_logout', 'loginGuest')
    expect(state.get('authenticated')).toBe(true)

    state.applyStep('demo_login_and_logout', 'logoutDemo')
    expect(state.get('authenticated')).toBe(false)
  })
})

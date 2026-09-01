import { describe, expect, it } from 'vitest'

import type { components } from '@/generated/api'

type ActiveRunResponse = components['schemas']['ActiveRunResponse']
type CorrectQueryResponse = components['schemas']['CorrectQueryResponse']
type IncorrectQueryResponse = components['schemas']['IncorrectQueryResponse']

const activeRunHasNoQueryCount: 'query_count' extends keyof ActiveRunResponse ? false : true = true

const elapsedIsNumber: ActiveRunResponse['elapsed_ms'] extends number ? true : false = true

const correctQueryCountIsNumber: CorrectQueryResponse['query_count'] extends number ? true : false =
  true

const incorrectQueryCountIsNumber: IncorrectQueryResponse['query_count'] extends number
  ? true
  : false = true

describe('generated API numeric contract', () => {
  it('keeps run and query counters aligned with the OpenAPI contract', () => {
    const activeRun: ActiveRunResponse = {
      status: 'active',
      started_at: '2026-08-06T10:00:00.000Z',
      elapsed_ms: 65_000,
      cleared_problem_ids: [],
    }

    const correctQuery: CorrectQueryResponse = {
      query_id: '33333333-3333-4333-8333-333333333333',
      correct: true,
      normalized_operations: [],
      remaining_pattern_count: 1,
      query_count: 2,
      problem_status: 'cleared',
    }

    const incorrectQuery: IncorrectQueryResponse = {
      query_id: '44444444-4444-4444-8444-444444444444',
      correct: false,
      normalized_operations: [],
      remaining_pattern_count: 2,
      query_count: 0,
      problem_status: 'available',
    }

    expect(activeRunHasNoQueryCount).toBe(true)
    expect(elapsedIsNumber).toBe(true)
    expect(correctQueryCountIsNumber).toBe(true)
    expect(incorrectQueryCountIsNumber).toBe(true)

    expect('query_count' in activeRun).toBe(false)
    expect(typeof activeRun.elapsed_ms).toBe('number')
    expect(typeof correctQuery.query_count).toBe('number')
    expect(typeof incorrectQuery.query_count).toBe('number')
  })
})

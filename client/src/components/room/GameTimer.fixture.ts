import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

import type { GameTimerProps } from './GameTimer.vue'

type ActiveRunResponse = components['schemas']['ActiveRunResponse']
type CorrectAnswerResponse = components['schemas']['CorrectAnswerResponse']

const activeRun = mockContract.getResponseExample(
  'getCurrentRun',
  200,
  'current_run',
) as ActiveRunResponse
const clearedRun = mockContract.getResponseExample(
  'submitAnswer',
  200,
  'correct_answer_clears_run',
) as CorrectAnswerResponse

export const gameTimerFixtures = {
  justStarted: {
    serverElapsedMs: 0,
  },
  active: {
    serverElapsedMs: activeRun.elapsed_ms,
  },
  cleared: {
    serverElapsedMs: clearedRun.elapsed_ms,
  },
} satisfies Record<string, GameTimerProps>

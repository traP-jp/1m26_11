import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

type AnswerResponse = components['schemas']['AnswerResponse']

function getAnswerFixture(
  example: 'incorrect_answer' | 'correct_answer_unlocks_problem',
): AnswerResponse {
  return mockContract.getResponseExample('submitAnswer', 200, example) as AnswerResponse
}

function judgeStateFromAnswer(response: AnswerResponse) {
  return response.correct ? ('correct' as const) : ('incorrect' as const)
}

const incorrectAnswer = getAnswerFixture('incorrect_answer')
const correctAnswer = getAnswerFixture('correct_answer_unlocks_problem')

export const answerJudgeResultFixtures = {
  idle: {
    state: 'idle',
  },
  pending: {
    state: 'pending',
  },
  correct: {
    state: judgeStateFromAnswer(correctAnswer),
  },
  incorrect: {
    state: judgeStateFromAnswer(incorrectAnswer),
  },
  error: {
    state: 'error',
  },
} as const

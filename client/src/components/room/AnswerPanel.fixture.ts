import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

type ProblemResponse = components['schemas']['ProblemResponse']
type AnswerRequest = components['schemas']['AnswerRequest']

const problem = mockContract.getResponseExample(
  'getProblem',
  200,
  'available_problem',
) as ProblemResponse
const submittedAnswer = mockContract.getRequestExample(
  'submitAnswer',
  'submitted_answer',
) as AnswerRequest

export const answerPanelFixture = {
  maxLength: problem.input_schema.answer.max_length,
  submittedAnswer: submittedAnswer.answer,
} as const

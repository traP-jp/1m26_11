import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

import type { ProblemBodyAssetsProps } from './ProblemBodyAssets.types'

type ProblemResponse = components['schemas']['ProblemResponse']

const problem = mockContract.getResponseExample(
  'getProblem',
  200,
  'available_problem',
) as ProblemResponse

export const problemBodyAssetsFixture = {
  bodyMarkdown: problem.body_markdown,
  assets: problem.assets,
} satisfies ProblemBodyAssetsProps

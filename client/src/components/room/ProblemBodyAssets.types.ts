import type { components } from '@/generated/api'

export type ProblemBodyAsset = components['schemas']['Asset']

type ProblemResponse = components['schemas']['ProblemResponse']

export interface ProblemBodyAssetsProps {
  bodyMarkdown: ProblemResponse['body_markdown']
  assets: readonly ProblemBodyAsset[]
}

import type { ProgressStatus } from './MinimalProgressSummary.vue'

export const minimalProgressSummaryFixtures = {
  notStarted: 'not_started',
  active: 'active',
  cleared: 'cleared',
} satisfies Record<string, ProgressStatus>

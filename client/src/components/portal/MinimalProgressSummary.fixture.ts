import type { ProgressStatus } from './MinimalProgressSummary.vue'

export const minimalProgressSummaryFixtures = {
  notStarted: 'not_started',
  inProgress: 'in_progress',
  cleared: 'cleared',
} satisfies Record<string, ProgressStatus>

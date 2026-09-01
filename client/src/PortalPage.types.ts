import type { Room } from './RoomCard.vue'
import type { ProgressStatus } from './components/portal/MinimalProgressSummary.vue'

export type PortalAuthMode = 'demo' | 'neoshowcase'

export interface PortalPageProps {
  authenticated: boolean
  authMode: PortalAuthMode
  displayName: string | null
  authBusy: boolean
  requiredRoom: Room
  progressStatus: ProgressStatus
}

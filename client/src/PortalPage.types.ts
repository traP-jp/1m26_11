import type { Room } from './RoomCard.vue'
import type { ProgressStatus } from './components/portal/MinimalProgressSummary.vue'

export type PortalAuthMode = 'demo' | 'neoshowcase'

export interface PortalRoom {
  room: Room
  progressStatus: ProgressStatus
}

export interface PortalPageProps {
  authenticated: boolean
  authMode: PortalAuthMode
  displayName: string | null
  authBusy: boolean
  loginHref: string | null
  logoutHref: string | null
  rooms: PortalRoom[]
  roomsLoading?: boolean
  roomsError?: string | null
}

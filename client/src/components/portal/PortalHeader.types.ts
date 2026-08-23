export type PortalAuthMode = 'demo' | 'neoshowcase'

export type PortalUserStatusState =
  | {
      authenticated: false
      authMode: PortalAuthMode
      loginHref: string | null
      loginPending: boolean
    }
  | {
      authenticated: true
      authMode: PortalAuthMode
      displayName: string
      logoutHref: string | null
      logoutPending: boolean
    }

export interface PortalHeaderProps {
  homeHref: string
  instructionsHref?: string | null
  userStatus: PortalUserStatusState
}

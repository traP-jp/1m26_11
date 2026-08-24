import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

import type { PortalHeaderProps, PortalUserStatusState } from './PortalHeader.types'

type MeResponse = components['schemas']['MeResponse']
type MeExample = 'demo_authenticated' | 'demo_unauthenticated' | 'neoshowcase_authenticated'

function getMeFixture(example: MeExample): MeResponse {
  return mockContract.getResponseExample('getMe', 200, example) as MeResponse
}

export function portalUserStatusFromMe(me: MeResponse, pending = false): PortalUserStatusState {
  if (!me.authenticated) {
    return {
      authenticated: false,
      authMode: me.auth_mode,
      loginHref: me.login_url,
      loginPending: pending,
    }
  }

  return {
    authenticated: true,
    authMode: me.auth_mode,
    displayName: me.user.display_name,
    logoutHref: me.logout_url,
    logoutPending: pending,
  }
}

const commonPaths = {
  homeHref: '/',
  instructionsHref: '#instructions',
} as const

export const portalHeaderFixtures = {
  unauthenticated: {
    ...commonPaths,
    userStatus: portalUserStatusFromMe(getMeFixture('demo_unauthenticated')),
  },
  demoAuthenticated: {
    ...commonPaths,
    userStatus: portalUserStatusFromMe(getMeFixture('demo_authenticated')),
  },
  neoshowcaseAuthenticated: {
    ...commonPaths,
    userStatus: portalUserStatusFromMe(getMeFixture('neoshowcase_authenticated')),
  },
} satisfies Record<string, PortalHeaderProps>

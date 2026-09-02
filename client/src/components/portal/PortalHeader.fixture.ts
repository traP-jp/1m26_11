import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'
import { portalUserStatusFromMe } from '@/utils/auth'

import type { PortalHeaderProps } from './PortalHeader.types'

type MeResponse = components['schemas']['MeResponse']
type MeExample = 'demo_authenticated' | 'demo_unauthenticated' | 'neoshowcase_authenticated'

function getMeFixture(example: MeExample): MeResponse {
  return mockContract.getResponseExample('getMe', 200, example) as MeResponse
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

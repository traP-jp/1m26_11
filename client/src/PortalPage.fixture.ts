import type { PortalPageProps } from './PortalPage.types'

const common = {
  authBusy: false,
  requiredRoom: {
    room_id: '1411824c-d357-4941-af76-c76cb827dda6',
    number: 1,
    name: '最初の部屋',
    genre: 'logic',
    description: '動作確認用の問題セットです',
  },
  progressStatus: 'not_started',
} as const

export const portalPageFixtures = {
  demoUnauthenticated: {
    ...common,
    authenticated: false,
    authMode: 'demo',
    displayName: null,
  },
  neoshowcaseUnauthenticated: {
    ...common,
    authenticated: false,
    authMode: 'neoshowcase',
    displayName: null,
  },
  demoAuthenticated: {
    ...common,
    authenticated: true,
    authMode: 'demo',
    displayName: 'kaomojikun',
    progressStatus: 'in_progress',
  },
  cleared: {
    ...common,
    authenticated: true,
    authMode: 'neoshowcase',
    displayName: 'kaomojikun',
    progressStatus: 'cleared',
  },
} satisfies Record<string, PortalPageProps>

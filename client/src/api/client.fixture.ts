import guestResponse from '../../../openapi/examples/auth/guest-response.json'
import meAuthenticated from '../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../openapi/examples/auth/me-demo-unauthenticated.json'

import type { ApiClient, GetMeResponse, LoginGuestResponse } from './client'

export function createFixtureApiClient(): ApiClient {
  let authenticated = true

  const unsupported = async (): Promise<never> => {
    throw new Error('このAPIはApp storyでは使用できません')
  }

  return {
    getMe: async () => (authenticated ? meAuthenticated : meUnauthenticated) as GetMeResponse,
    loginGuest: async () => {
      authenticated = true
      return guestResponse as LoginGuestResponse
    },
    logoutDemo: async () => {
      authenticated = false
    },
    startOrResumeRun: unsupported,
    getCurrentRun: unsupported,
    getProblem: unsupported,
    submitQuery: unsupported,
    submitAnswer: unsupported,
  }
}

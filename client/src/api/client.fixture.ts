import guestResponse from '../../../openapi/examples/auth/guest-response.json'
import meAuthenticated from '../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../openapi/examples/auth/me-demo-unauthenticated.json'
import roomsResponse from '../../../openapi/examples/rooms/response-authenticated-active.json'

import type { ApiClient, GetMeResponse, GetRoomsResponse, LoginGuestResponse } from './client'

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
    getRooms: async () => roomsResponse as GetRoomsResponse,
    getRoom: unsupported,
    startOrResumeRun: unsupported,
    getCurrentRun: unsupported,
    getProblems: unsupported,
    getProblem: unsupported,
    getProblemAssets: unsupported,
    submitQuery: unsupported,
    submitAnswer: unsupported,
  }
}

import { describe, expect, it } from 'vitest'
import problemAssetCreatedFixture from '../../../../openapi/examples/assets/response-created.json'
import leaderboardEmptyFixture from '../../../../openapi/examples/leaderboard/response-empty.json'
import leaderboardRankedFixture from '../../../../openapi/examples/leaderboard/response-ranked.json'
import leaderboardUnauthenticatedFixture from '../../../../openapi/examples/leaderboard/response-unauthenticated.json'
import meProgressEmptyFixture from '../../../../openapi/examples/progress/response-empty.json'
import meProgressSummaryFixture from '../../../../openapi/examples/progress/response-summary.json'

import type { components, operations } from '@/generated/api'

type ActiveRunResponse = components['schemas']['ActiveRunResponse']
type CorrectQueryResponse = components['schemas']['CorrectQueryResponse']
type IncorrectQueryResponse = components['schemas']['IncorrectQueryResponse']
type LeaderboardResponse = components['schemas']['LeaderboardResponse']
type MeProgressResponse = components['schemas']['MeProgressResponse']
type Asset = components['schemas']['Asset']
type UploadProblemAssetOperation = operations['uploadProblemAsset']
type UploadProblemAssetBody =
  UploadProblemAssetOperation['requestBody']['content']['multipart/form-data']
type UploadProblemAssetHeaders = UploadProblemAssetOperation['parameters']['header']

const leaderboardMeAcceptsNull: null extends LeaderboardResponse['me'] ? true : false = true
const summaryProgress: MeProgressResponse = meProgressSummaryFixture
const emptyProgress: MeProgressResponse = meProgressEmptyFixture

type LeaderboardMeIsRequired =
  Pick<LeaderboardResponse, 'me'> extends Required<Pick<LeaderboardResponse, 'me'>> ? true : false

const leaderboardMeIsRequired: LeaderboardMeIsRequired = true

const rankedLeaderboard: LeaderboardResponse = leaderboardRankedFixture
const unauthenticatedLeaderboard: LeaderboardResponse = leaderboardUnauthenticatedFixture
const emptyLeaderboard: LeaderboardResponse = leaderboardEmptyFixture

const activeRunHasNoQueryCount: 'query_count' extends keyof ActiveRunResponse ? false : true = true

const elapsedIsNumber: ActiveRunResponse['elapsed_ms'] extends number ? true : false = true

const correctQueryCountIsNumber: CorrectQueryResponse['query_count'] extends number ? true : false =
  true

const incorrectQueryCountIsNumber: IncorrectQueryResponse['query_count'] extends number
  ? true
  : false = true

const uploadedAsset: Asset = problemAssetCreatedFixture

const uploadBody: UploadProblemAssetBody = {
  file: 'binary image placeholder',
  alt: uploadedAsset.alt,
}

const uploadHeaders: UploadProblemAssetHeaders = {
  'Idempotency-Key': '44444444-4444-4444-8444-444444444444',
}

describe('generated API contract', () => {
  it('keeps run and query counters aligned with the OpenAPI contract', () => {
    const activeRun: ActiveRunResponse = {
      status: 'active',
      started_at: '2026-08-06T10:00:00.000Z',
      elapsed_ms: 65_000,
      cleared_problem_ids: [],
    }

    const correctQuery: CorrectQueryResponse = {
      query_id: '33333333-3333-4333-8333-333333333333',
      correct: true,
      normalized_operations: [],
      remaining_pattern_count: 1,
      query_count: 2,
      problem_status: 'cleared',
    }

    const incorrectQuery: IncorrectQueryResponse = {
      query_id: '44444444-4444-4444-8444-444444444444',
      correct: false,
      normalized_operations: [],
      remaining_pattern_count: 2,
      query_count: 0,
      problem_status: 'available',
    }

    expect(activeRunHasNoQueryCount).toBe(true)
    expect(elapsedIsNumber).toBe(true)
    expect(correctQueryCountIsNumber).toBe(true)
    expect(incorrectQueryCountIsNumber).toBe(true)

    expect('query_count' in activeRun).toBe(false)
    expect(typeof activeRun.elapsed_ms).toBe('number')
    expect(typeof correctQuery.query_count).toBe('number')
    expect(typeof incorrectQuery.query_count).toBe('number')
  })

  it('keeps leaderboard fixtures aligned with the generated contract', () => {
    expect(leaderboardMeAcceptsNull).toBe(true)
    expect(leaderboardMeIsRequired).toBe(true)

    expect(rankedLeaderboard.entries.map((entry) => entry.rank)).toEqual([1, 1, 3])
    expect(rankedLeaderboard.me).toEqual({
      rank: 3,
      elapsed_ms: 80_000,
      query_count: 18,
    })

    expect(unauthenticatedLeaderboard.me).toBeNull()
    expect(emptyLeaderboard.entries).toEqual([])
    expect(emptyLeaderboard.me).toBeNull()
  })

  it('keeps progress fixtures aligned with the generated contract', () => {
    expect(summaryProgress.cleared_room_count).toBe(5)
    expect(summaryProgress.total_room_count).toBe(20)

    expect(summaryProgress.by_genre.map((progress) => progress.genre)).toEqual(['OSINT', 'Web'])

    const clearedRoomCount = summaryProgress.by_genre.reduce(
      (total, progress) => total + progress.cleared_room_count,
      0,
    )

    const totalRoomCount = summaryProgress.by_genre.reduce(
      (total, progress) => total + progress.total_room_count,
      0,
    )

    expect(clearedRoomCount).toBe(summaryProgress.cleared_room_count)
    expect(totalRoomCount).toBe(summaryProgress.total_room_count)

    expect(emptyProgress).toEqual({
      cleared_room_count: 0,
      total_room_count: 0,
      by_genre: [],
    })
  })
  it('keeps problem asset upload aligned with the generated contract', () => {
    expect(uploadedAsset).toEqual(problemAssetCreatedFixture)
    expect(uploadedAsset.type).toBe('image')
    expect(uploadedAsset.url).toContain('/v1/problems/')
    expect('object_key' in uploadedAsset).toBe(false)

    expect(uploadBody.file).toBe('binary image placeholder')
    expect(uploadBody.alt).toBe(problemAssetCreatedFixture.alt)
    expect(uploadHeaders['Idempotency-Key']).toBe('44444444-4444-4444-8444-444444444444')
  })
})

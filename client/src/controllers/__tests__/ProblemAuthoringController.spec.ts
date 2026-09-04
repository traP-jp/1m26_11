import { describe, expect, it, vi } from 'vitest'

import assetCreated from '../../../../openapi/examples/assets/response-created.json'
import createProblemResult from '../../../../openapi/examples/problems/create-response.json'
import createStringProblem from '../../../../openapi/examples/problems/create-string-request.json'
import type {
  ApiClient,
  CreateProblemRequest,
  CreateProblemResponse,
  UploadProblemAssetResponse,
} from '@/api/client'

import {
  ProblemAuthoringController,
  type IdempotencyKeyFactory,
} from '../ProblemAuthoringController'

const ROOM_ID = '11111111-1111-4111-8111-111111111111'
const CREATE_KEY = '44444444-4444-4444-8444-444444444441'
const UPLOAD_KEY = '44444444-4444-4444-8444-444444444442'

const STRING_REQUEST = createStringProblem as CreateProblemRequest
const CREATE_RESPONSE = createProblemResult as CreateProblemResponse
const UPLOAD_RESPONSE = assetCreated as UploadProblemAssetResponse

function createClient(overrides: Partial<ApiClient> = {}): ApiClient {
  return {
    getMe: vi.fn<ApiClient['getMe']>(),
    loginGuest: vi.fn<ApiClient['loginGuest']>(),
    logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
    startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
    getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
    getProblem: vi.fn<ApiClient['getProblem']>(),
    submitQuery: vi.fn<ApiClient['submitQuery']>(),
    submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
    createProblem: vi.fn<ApiClient['createProblem']>(),
    uploadProblemAsset: vi.fn<ApiClient['uploadProblemAsset']>(),
    ...overrides,
  }
}

function sequentialKeys(...keys: string[]): IdempotencyKeyFactory {
  let index = 0

  return () => {
    const key = keys[index]
    if (key === undefined) {
      throw new Error('テスト用Idempotency-Keyが不足しています')
    }
    index += 1
    return key
  }
}

describe('ProblemAuthoringController', () => {
  it('creates a problem without uploading an image', async () => {
    const createProblem = vi.fn<ApiClient['createProblem']>().mockResolvedValue(CREATE_RESPONSE)
    const uploadProblemAsset = vi.fn<ApiClient['uploadProblemAsset']>()
    const controller = new ProblemAuthoringController(
      createClient({ createProblem, uploadProblemAsset }),
      sequentialKeys(CREATE_KEY),
    )
    const request = structuredClone(STRING_REQUEST)

    const result = await controller.submit(ROOM_ID, request, null)

    expect(result).toEqual(CREATE_RESPONSE)
    expect(createProblem).toHaveBeenCalledExactlyOnceWith(
      { room_id: ROOM_ID },
      { 'Idempotency-Key': CREATE_KEY },
      request,
    )
    expect(uploadProblemAsset).not.toHaveBeenCalled()
    expect(controller.state).toMatchObject({
      phase: 'succeeded',
      roomId: ROOM_ID,
      problemId: CREATE_RESPONSE.problem_id,
      imageSelected: false,
      imageUploaded: false,
      errorStage: null,
      error: null,
    })
  })

  it('uploads the selected image only after creating the problem', async () => {
    const createProblem = vi.fn<ApiClient['createProblem']>().mockResolvedValue(CREATE_RESPONSE)
    const uploadProblemAsset = vi
      .fn<ApiClient['uploadProblemAsset']>()
      .mockResolvedValue(UPLOAD_RESPONSE)
    const controller = new ProblemAuthoringController(
      createClient({ createProblem, uploadProblemAsset }),
      sequentialKeys(CREATE_KEY, UPLOAD_KEY),
    )
    const file = new File(['image'], 'question.png', { type: 'image/png' })

    await controller.submit(ROOM_ID, structuredClone(STRING_REQUEST), {
      file,
      alt: '問題画像',
    })

    expect(createProblem).toHaveBeenCalledOnce()
    expect(uploadProblemAsset).toHaveBeenCalledExactlyOnceWith(
      {
        room_id: ROOM_ID,
        problem_id: CREATE_RESPONSE.problem_id,
      },
      { 'Idempotency-Key': UPLOAD_KEY },
      {
        file,
        alt: '問題画像',
      },
    )
    expect(createProblem.mock.invocationCallOrder[0]).toBeLessThan(
      uploadProblemAsset.mock.invocationCallOrder[0] ?? 0,
    )
    expect(controller.state).toMatchObject({
      phase: 'succeeded',
      problemId: CREATE_RESPONSE.problem_id,
      imageSelected: true,
      imageUploaded: true,
      error: null,
    })
  })

  it('retries problem creation with the same key and request snapshot', async () => {
    const createProblem = vi
      .fn<ApiClient['createProblem']>()
      .mockRejectedValueOnce(new Error('connection lost'))
      .mockResolvedValue(CREATE_RESPONSE)
    const keyFactory = vi.fn<IdempotencyKeyFactory>().mockImplementation(sequentialKeys(CREATE_KEY))
    const controller = new ProblemAuthoringController(createClient({ createProblem }), keyFactory)
    const request = structuredClone(STRING_REQUEST)
    const submittedTitle = request.title

    await expect(controller.submit(ROOM_ID, request, null)).rejects.toMatchObject({
      kind: 'network',
    })

    request.title = '再試行前に変更されたタイトル'

    expect(controller.state).toMatchObject({
      phase: 'create-error',
      problemId: null,
      errorStage: 'create',
    })

    await controller.retry()

    expect(createProblem).toHaveBeenCalledTimes(2)
    expect(createProblem.mock.calls[1]?.[1]).toEqual({
      'Idempotency-Key': CREATE_KEY,
    })
    expect(createProblem.mock.calls[1]?.[2].title).toBe(submittedTitle)
    expect(keyFactory).toHaveBeenCalledOnce()
    expect(controller.state.phase).toBe('succeeded')
  })

  it('retries only the image upload after the problem was created', async () => {
    const createProblem = vi.fn<ApiClient['createProblem']>().mockResolvedValue(CREATE_RESPONSE)
    const uploadProblemAsset = vi
      .fn<ApiClient['uploadProblemAsset']>()
      .mockRejectedValueOnce(new Error('upload failed'))
      .mockResolvedValue(UPLOAD_RESPONSE)
    const keyFactory = vi
      .fn<IdempotencyKeyFactory>()
      .mockImplementation(sequentialKeys(CREATE_KEY, UPLOAD_KEY))
    const controller = new ProblemAuthoringController(
      createClient({ createProblem, uploadProblemAsset }),
      keyFactory,
    )
    const file = new File(['image'], 'question.png', { type: 'image/png' })

    await expect(
      controller.submit(ROOM_ID, structuredClone(STRING_REQUEST), {
        file,
        alt: '問題画像',
      }),
    ).rejects.toMatchObject({ kind: 'network' })

    expect(controller.state).toMatchObject({
      phase: 'upload-error',
      problemId: CREATE_RESPONSE.problem_id,
      imageUploaded: false,
      errorStage: 'upload',
    })

    await controller.retry()

    expect(createProblem).toHaveBeenCalledOnce()
    expect(uploadProblemAsset).toHaveBeenCalledTimes(2)
    expect(uploadProblemAsset.mock.calls[0]?.[1]).toEqual({
      'Idempotency-Key': UPLOAD_KEY,
    })
    expect(uploadProblemAsset.mock.calls[1]?.[1]).toEqual({
      'Idempotency-Key': UPLOAD_KEY,
    })
    expect(keyFactory).toHaveBeenCalledTimes(2)
    expect(controller.state).toMatchObject({
      phase: 'succeeded',
      problemId: CREATE_RESPONSE.problem_id,
      imageUploaded: true,
      errorStage: null,
      error: null,
    })
  })
})

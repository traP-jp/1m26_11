import { reactive } from 'vue'

import {
  ApiClientError,
  type ApiClient,
  type CreateProblemRequest,
  type CreateProblemResponse,
} from '@/api/client'

export type ProblemAuthoringPhase =
  'idle' | 'creating' | 'uploading' | 'succeeded' | 'create-error' | 'upload-error'

export type ProblemAuthoringErrorStage = 'create' | 'upload' | null

export interface ProblemAssetDraft {
  file: File
  alt: string
}

export interface ProblemAuthoringControllerState {
  phase: ProblemAuthoringPhase
  roomId: string | null
  problemId: string | null
  imageSelected: boolean
  imageUploaded: boolean
  errorStage: ProblemAuthoringErrorStage
  error: ApiClientError | null
}

export type IdempotencyKeyFactory = () => string

interface PendingSubmission {
  roomId: string
  request: CreateProblemRequest
  image: ProblemAssetDraft | null
  createIdempotencyKey: string
  uploadIdempotencyKey: string | null
}

function normalizeError(error: unknown): ApiClientError {
  if (error instanceof ApiClientError) return error

  return new ApiClientError('APIとの通信に失敗しました', {
    kind: 'network',
    cause: error,
  })
}

function cloneRequest(request: CreateProblemRequest): CreateProblemRequest {
  return structuredClone(request)
}

function isBusy(phase: ProblemAuthoringPhase): boolean {
  return phase === 'creating' || phase === 'uploading'
}

export class ProblemAuthoringController {
  readonly state = reactive<ProblemAuthoringControllerState>({
    phase: 'idle',
    roomId: null,
    problemId: null,
    imageSelected: false,
    imageUploaded: false,
    errorStage: null,
    error: null,
  })

  private pending: PendingSubmission | null = null

  constructor(
    private readonly client: ApiClient,
    private readonly createIdempotencyKey: IdempotencyKeyFactory = () => crypto.randomUUID(),
  ) {}

  async submit(
    roomId: string,
    request: CreateProblemRequest,
    image: ProblemAssetDraft | null,
  ): Promise<CreateProblemResponse | null> {
    if (isBusy(this.state.phase)) return null

    this.pending = {
      roomId,
      request: cloneRequest(request),
      image: image ? { file: image.file, alt: image.alt } : null,
      createIdempotencyKey: this.createIdempotencyKey(),
      uploadIdempotencyKey: image ? this.createIdempotencyKey() : null,
    }

    Object.assign(this.state, {
      phase: 'creating',
      roomId,
      problemId: null,
      imageSelected: image !== null,
      imageUploaded: false,
      errorStage: null,
      error: null,
    })

    return this.createPendingProblem()
  }

  async retry(): Promise<CreateProblemResponse | null> {
    const pending = this.pending
    if (!pending || isBusy(this.state.phase)) return null

    if (this.state.phase === 'create-error') {
      return this.createPendingProblem()
    }

    if (
      this.state.phase === 'upload-error' &&
      pending.image !== null &&
      this.state.problemId !== null
    ) {
      await this.uploadPendingImage(pending, this.state.problemId)
      return { problem_id: this.state.problemId }
    }

    return null
  }

  private async createPendingProblem(): Promise<CreateProblemResponse> {
    const pending = this.pending
    if (!pending) {
      throw new Error('送信対象の問題がありません')
    }

    this.state.phase = 'creating'
    this.state.errorStage = null
    this.state.error = null

    let response: CreateProblemResponse
    try {
      response = await this.client.createProblem(
        { room_id: pending.roomId },
        { 'Idempotency-Key': pending.createIdempotencyKey },
        pending.request,
      )
    } catch (error) {
      const normalized = normalizeError(error)
      this.state.phase = 'create-error'
      this.state.errorStage = 'create'
      this.state.error = normalized
      throw normalized
    }

    this.state.problemId = response.problem_id

    if (pending.image === null) {
      this.state.phase = 'succeeded'
      return response
    }

    await this.uploadPendingImage(pending, response.problem_id)
    return response
  }

  private async uploadPendingImage(pending: PendingSubmission, problemId: string): Promise<void> {
    if (pending.image === null || pending.uploadIdempotencyKey === null) {
      this.state.phase = 'succeeded'
      return
    }

    this.state.phase = 'uploading'
    this.state.errorStage = null
    this.state.error = null

    try {
      await this.client.uploadProblemAsset(
        {
          room_id: pending.roomId,
          problem_id: problemId,
        },
        { 'Idempotency-Key': pending.uploadIdempotencyKey },
        pending.image,
      )
    } catch (error) {
      const normalized = normalizeError(error)
      this.state.phase = 'upload-error'
      this.state.errorStage = 'upload'
      this.state.error = normalized
      throw normalized
    }

    this.state.imageUploaded = true
    this.state.phase = 'succeeded'
  }
}

import type { Control, InputAdapterEvent, Operation } from './input/InputAdapter.types'
import type { components } from './generated/api'

export type ProblemStatus = 'locked' | 'available' | 'cleared'
export type ProblemType = 'small' | 'final'
export type JudgementState = 'idle' | 'pending' | 'correct' | 'incorrect' | 'error'

export interface RoomViewModel {
  room: { id: string; number: number; name: string }
  problems: Array<{
    id: string
    number: number
    title: string
    status: ProblemStatus
    selected: boolean
  }>
  selectedProblem: {
    id: string
    number: number
    type: ProblemType
    submissionType: components['schemas']['SubmissionType']
    title: string
    bodyMarkdown: string
    assets: Array<{ type: string; url: string; alt: string }>
    hintCount: number
  } | null
  serverElapsedMs: number
  queryInput: {
    allowedControls: Control[]
    maxOperations: number
    operations: Operation[]
  }
  answerInput: { value: string; maxLength: number }
  queryJudgement: { state: JudgementState }
  answerJudgement: { state: JudgementState }
  clear: { cleared: boolean; clearedCount: number; requiredCount: number }
}

export type RoomUiEvent =
  | { type: 'problem-selected'; problemId: string }
  | { type: 'query-operation-removed'; index: number }
  | { type: 'query-operations-cleared' }
  | { type: 'answer-changed'; value: string }
  | { type: 'portal-returned' }
  | { type: 'room-exited' }
  | InputAdapterEvent

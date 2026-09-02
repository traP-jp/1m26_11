import type { InputAdapterEvent } from './input/InputAdapter.types'

export type ProblemStatus = 'locked' | 'available' | 'cleared'
export type JudgementState = 'idle' | 'pending' | 'correct' | 'incorrect' | 'error'

export interface RoomViewModel {
  room: { id: string; name: string }
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
    title: string
    bodyMarkdown: string
    assets: Array<{ type: string; url: string; alt: string }>
    hintCount: number
  } | null
  serverElapsedMs: number
  queryInput: {
    allowedControls: string[]
    maxOperations: number
    operations: Array<{ control: string; count: number }>
  }
  answerInput: { value: string; maxLength: number }
  queryJudgement: { state: JudgementState }
  answerJudgement: { state: JudgementState }
  clear: { cleared: boolean; clearedProblemCount: number; totalProblemCount: number }
}

export type RoomUiEvent =
  | { type: 'problem-selected'; problemId: string }
  | { type: 'answer-changed'; value: string }
  | { type: 'room-exited' }
  | InputAdapterEvent

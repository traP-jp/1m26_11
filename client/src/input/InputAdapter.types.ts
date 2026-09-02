import type { SubmitQueryRequest } from '@/api/client'

export type Operation = SubmitQueryRequest['operations'][number]
export type Control = Operation['control']
/** Kept as the generated API type until the allowed source values are finalized. */
export type InputSource = SubmitQueryRequest['source']

export const keyboardInputSource = 'keyboard' satisfies InputSource
/** Screen-button query submissions use the backend's existing `mouse` source value. */
export const screenButtonInputSource = 'mouse' satisfies InputSource
export type AlternativeInputSource = typeof keyboardInputSource | typeof screenButtonInputSource

export type InputAdapterEvent =
  | {
      type: 'condition-changed'
      source: InputSource
      control: Control
      count: Operation['count']
    }
  | { type: 'query-submitted'; source: InputSource }
  | { type: 'answer-submitted'; source: InputSource; answer: string }

/** Resolves only after downstream handling, including any submission request, has settled. */
export type InputAdapterEventHandler = (event: InputAdapterEvent) => Promise<void>

export interface InputAdapterDispatcher {
  readonly busy: boolean
  /** Returns whether the event was accepted by the guards. */
  dispatch(event: InputAdapterEvent): boolean
}

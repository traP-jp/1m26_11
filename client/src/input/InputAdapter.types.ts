import type { SubmitQueryRequest } from '@/api/client'

export type Operation = SubmitQueryRequest['operations'][number]
export type Control = Operation['control']
/** Kept as the generated API type until the allowed source values are finalized. */
export type InputSource = SubmitQueryRequest['source']

export type InputAdapterEvent =
  | {
      type: 'condition-changed'
      source: InputSource
      control: Control
      count: Operation['count']
    }
  | { type: 'query-submitted'; source: InputSource }
  | { type: 'answer-submitted'; source: InputSource }

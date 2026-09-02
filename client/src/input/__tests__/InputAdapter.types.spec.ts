import { describe, expect, expectTypeOf, it } from 'vitest'

import type { SubmitQueryRequest } from '@/api/client'
import type { RoomUiEvent } from '@/RoomPage.types'
import { createOperationBuffer } from '../operationBuffer'
import type { Control, InputAdapterEvent, InputSource, Operation } from '../InputAdapter.types'

type ConditionChangedEvent = Extract<InputAdapterEvent, { type: 'condition-changed' }>
type QuerySubmittedEvent = Extract<InputAdapterEvent, { type: 'query-submitted' }>
type AnswerSubmittedEvent = Extract<InputAdapterEvent, { type: 'answer-submitted' }>

describe('InputAdapterEvent', () => {
  it('uses the generated query contract for source, control, and operations', () => {
    expectTypeOf<InputSource>().toEqualTypeOf<SubmitQueryRequest['source']>()
    expectTypeOf<Control>().toEqualTypeOf<SubmitQueryRequest['operations'][number]['control']>()
    expectTypeOf<Operation>().toEqualTypeOf<SubmitQueryRequest['operations'][number]>()
  })

  it('uses only the semantic Room UI event fields and input source', () => {
    expectTypeOf<InputAdapterEvent>().toMatchTypeOf<RoomUiEvent>()
    expectTypeOf<ConditionChangedEvent>().toEqualTypeOf<{
      type: 'condition-changed'
      source: InputSource
      control: Control
      count: Operation['count']
    }>()
    expectTypeOf<QuerySubmittedEvent>().toEqualTypeOf<{
      type: 'query-submitted'
      source: InputSource
    }>()
    expectTypeOf<AnswerSubmittedEvent>().toEqualTypeOf<{
      type: 'answer-submitted'
      source: InputSource
    }>()

    const events = [
      { type: 'query-submitted', source: 'serial' },
      { type: 'answer-submitted', source: 'keyboard' },
    ] satisfies InputAdapterEvent[]

    expect(events).toEqual([
      { type: 'query-submitted', source: 'serial' },
      { type: 'answer-submitted', source: 'keyboard' },
    ])
  })

  it('converts the currently accepted query sources through one operation event shape', () => {
    const sources = ['serial', 'keyboard', 'mouse'] satisfies InputSource[]
    const events: InputAdapterEvent[] = sources.map((source) => ({
      type: 'condition-changed',
      source,
      control: 'up',
      count: 1,
    }))

    const snapshots = events.map((event) => {
      const buffer = createOperationBuffer()
      if (event.type === 'condition-changed') buffer.append(event)
      return buffer.snapshot()
    })

    expect(snapshots).toEqual(sources.map(() => [{ control: 'up', count: 1 }]))
  })
})

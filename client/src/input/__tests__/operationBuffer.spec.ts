import { describe, expect, expectTypeOf, it, vi } from 'vitest'

import type { SubmitQueryRequest } from '@/api/client'
import { createOperationBuffer, type OperationSnapshot } from '../operationBuffer'

describe('createOperationBuffer', () => {
  it('starts empty and safely clears an empty snapshot', () => {
    const buffer = createOperationBuffer()

    expect(buffer.snapshot()).toEqual([])
    expect(buffer.clear(buffer.snapshot())).toEqual([])
    expect(buffer.snapshot()).toEqual([])
  })

  it('appends operations and aggregates only adjacent matching controls', () => {
    const buffer = createOperationBuffer()

    buffer.append({ control: 'down', count: 1 })
    buffer.append({ control: 'down', count: 2 })
    buffer.append({ control: 'right', count: 1 })
    buffer.append({ control: 'down', count: 1 })

    expect(buffer.snapshot()).toEqual([
      { control: 'down', count: 3 },
      { control: 'right', count: 1 },
      { control: 'down', count: 1 },
    ])
  })

  it('copies appended objects and returns an API-compatible frozen snapshot', () => {
    const buffer = createOperationBuffer()
    const operation = { control: 'up', count: 2 }

    buffer.append(operation)
    operation.control = 'down'
    operation.count = 20

    const snapshot = buffer.snapshot()
    const firstOperation = snapshot[0]
    if (firstOperation === undefined) throw new Error('expected a buffered operation')

    expectTypeOf(snapshot).toEqualTypeOf<OperationSnapshot>()
    expectTypeOf(snapshot).toMatchTypeOf<SubmitQueryRequest['operations']>()
    expect(Object.isFrozen(snapshot)).toBe(true)
    expect(Object.isFrozen(firstOperation)).toBe(true)
    expect(() => {
      firstOperation.control = 'left'
    }).toThrow(TypeError)
    expect(() => {
      snapshot.push({ control: 'right', count: 1 })
    }).toThrow(TypeError)
    expect(buffer.snapshot()).toEqual([{ control: 'up', count: 2 }])
    expect(buffer.clear(snapshot)).toEqual([])
  })

  it('strips adapter-only data from operations appended as structural subtypes', () => {
    const buffer = createOperationBuffer()
    const adapterPayload = {
      control: 'up',
      count: 1,
      source: 'serial',
      rawData: new Uint8Array([0x01]),
    }

    buffer.append(adapterPayload)

    expect(buffer.snapshot()).toEqual([{ control: 'up', count: 1 }])
  })

  it('clears the successfully submitted snapshot', () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'down', count: 2 })
    buffer.append({ control: 'right', count: 1 })

    const submitted = buffer.snapshot()

    expect(buffer.clear(submitted)).toEqual([])
    expect(buffer.snapshot()).toEqual([])
  })

  it('preserves operations appended while a snapshot is being submitted', () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'down', count: 2 })
    const submitted = buffer.snapshot()

    buffer.append({ control: 'right', count: 1 })

    expect(buffer.clear(submitted)).toEqual([{ control: 'right', count: 1 }])
    expect(buffer.snapshot()).toEqual([{ control: 'right', count: 1 }])
  })

  it('preserves an appended count when it extends the submitted tail control', () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'down', count: 2 })
    const submitted = buffer.snapshot()

    buffer.append({ control: 'down', count: 1 })

    expect(buffer.clear(submitted)).toEqual([{ control: 'down', count: 1 }])
  })

  it('retains the submitted snapshot and later input when submission fails', async () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'down', count: 2 })
    const submitted = buffer.snapshot()
    const submit = vi.fn<(operations: SubmitQueryRequest['operations']) => Promise<void>>(
      async () => {
        buffer.append({ control: 'right', count: 1 })
        throw new Error('query failed')
      },
    )

    await expect(submit(submitted).then(() => buffer.clear(submitted))).rejects.toThrow(
      'query failed',
    )

    expect(submit).toHaveBeenCalledExactlyOnceWith([{ control: 'down', count: 2 }])
    expect(submitted).toEqual([{ control: 'down', count: 2 }])
    expect(buffer.snapshot()).toEqual([
      { control: 'down', count: 2 },
      { control: 'right', count: 1 },
    ])
  })

  it.each([1.5, Number.NaN, Number.POSITIVE_INFINITY])(
    'rejects a non-integer operation count: %s',
    (count) => {
      const buffer = createOperationBuffer()

      expect(() => buffer.append({ control: 'up', count })).toThrow(RangeError)
      expect(buffer.snapshot()).toEqual([])
    },
  )

  it('rejects a cleared snapshot even when later input has the same value', () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'up', count: 1 })
    const submitted = buffer.snapshot()
    buffer.clear(submitted)
    buffer.append({ control: 'up', count: 1 })

    expect(() => buffer.clear(submitted)).toThrow(
      'snapshot does not belong to this buffer or has already been cleared',
    )
    expect(buffer.snapshot()).toEqual([{ control: 'up', count: 1 }])
  })

  it('rejects a snapshot created by another buffer', () => {
    const firstBuffer = createOperationBuffer()
    const secondBuffer = createOperationBuffer()
    firstBuffer.append({ control: 'up', count: 1 })
    secondBuffer.append({ control: 'up', count: 1 })

    expect(() => secondBuffer.clear(firstBuffer.snapshot())).toThrow(
      'snapshot does not belong to this buffer or has already been cleared',
    )
    expect(secondBuffer.snapshot()).toEqual([{ control: 'up', count: 1 }])
  })

  it('rejects another snapshot of a prefix that was already cleared', () => {
    const buffer = createOperationBuffer()
    buffer.append({ control: 'up', count: 1 })
    const firstSnapshot = buffer.snapshot()
    const secondSnapshot = buffer.snapshot()

    buffer.clear(firstSnapshot)

    expect(() => buffer.clear(secondSnapshot)).toThrow('snapshot is stale')
    expect(buffer.snapshot()).toEqual([])
  })
})

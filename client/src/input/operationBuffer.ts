import type { Operation } from './InputAdapter.types'

declare const operationSnapshotBrand: unique symbol

/** An API-compatible operations array that is frozen at runtime and cleared by identity. */
export type OperationSnapshot = Operation[] & {
  readonly [operationSnapshotBrand]: true
}

/** Buffers API operations only; the submit controller owns request-level source selection. */
export interface OperationBuffer {
  append(operation: Operation): void
  snapshot(): OperationSnapshot
  /** Removes only the operations captured by this exact snapshot. */
  clear(snapshot: OperationSnapshot): Operation[]
}

interface BufferedOperation extends Operation {
  sequence: number
}

interface SnapshotMetadata {
  boundarySequence: number | undefined
  operations: Operation[]
  cleared: boolean
}

function normalizeOperations(operations: readonly Operation[]): Operation[] {
  const normalized: Operation[] = []

  for (const { control, count } of operations) {
    const previous = normalized[normalized.length - 1]
    if (previous?.control === control) {
      previous.count += count
    } else {
      normalized.push({ control, count })
    }
  }

  return normalized
}

function operationsEqual(left: readonly Operation[], right: readonly Operation[]): boolean {
  if (left.length !== right.length) return false

  for (let index = 0; index < left.length; index += 1) {
    const leftOperation = left[index]
    const rightOperation = right[index]
    if (
      leftOperation === undefined ||
      rightOperation === undefined ||
      leftOperation.control !== rightOperation.control ||
      leftOperation.count !== rightOperation.count
    ) {
      return false
    }
  }

  return true
}

function validateCount(count: number): void {
  if (!Number.isInteger(count)) {
    throw new RangeError('operation count must be an integer')
  }
}

function freezeSnapshot(operations: Operation[]): OperationSnapshot {
  for (const operation of operations) Object.freeze(operation)
  return Object.freeze(operations) as OperationSnapshot
}

export function createOperationBuffer(): OperationBuffer {
  let operations: BufferedOperation[] = []
  let nextSequence = 0
  const snapshots = new WeakMap<readonly Operation[], SnapshotMetadata>()

  return {
    append({ control, count }) {
      validateCount(count)
      operations.push({ control, count, sequence: nextSequence })
      nextSequence += 1
    },

    snapshot() {
      const snapshot = freezeSnapshot(normalizeOperations(operations))
      snapshots.set(snapshot, {
        boundarySequence: operations[operations.length - 1]?.sequence,
        operations: normalizeOperations(operations),
        cleared: false,
      })
      return snapshot
    },

    clear(snapshot) {
      const metadata = snapshots.get(snapshot)
      if (metadata === undefined || metadata.cleared) {
        throw new Error('snapshot does not belong to this buffer or has already been cleared')
      }
      if (!operationsEqual(snapshot, metadata.operations)) {
        throw new Error('snapshot has been modified')
      }

      const { boundarySequence } = metadata
      if (boundarySequence !== undefined) {
        const boundaryIndex = operations.findIndex(
          (operation) => operation.sequence === boundarySequence,
        )
        if (boundaryIndex === -1) {
          throw new Error('snapshot is stale')
        }

        operations = operations.slice(boundaryIndex + 1)
      }

      metadata.cleared = true

      return normalizeOperations(operations)
    },
  }
}

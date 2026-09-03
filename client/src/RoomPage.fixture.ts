import type { components } from '@/generated/api'
import { mockContract } from '@/mocks/data'

import type { RoomViewModel } from './RoomPage.types'

type ActiveRunResponse = components['schemas']['ActiveRunResponse']
type ProblemResponse = components['schemas']['ProblemResponse']
type QueryRequest = components['schemas']['QueryRequest']

interface RoomFixture {
  room_id: string
  number: number
  name: string
  problem_count: number
  required_count: number
}

export function roomViewModelFromFixtures(
  room: RoomFixture,
  run: ActiveRunResponse,
  problem: ProblemResponse,
  query: QueryRequest,
): RoomViewModel {
  return {
    room: { id: room.room_id, number: room.number, name: room.name },
    problems: [
      {
        id: problem.id,
        number: problem.number,
        title: problem.title,
        status: problem.status,
        selected: true,
      },
    ],
    selectedProblem: {
      id: problem.id,
      number: problem.number,
      type: problem.type,
      title: problem.title,
      bodyMarkdown: problem.body_markdown,
      assets: problem.assets,
      hintCount: problem.hint_count,
    },
    serverElapsedMs: run.elapsed_ms,
    queryInput: {
      allowedControls: problem.input_schema.query.allowed_controls,
      maxOperations: problem.input_schema.query.max_operations,
      operations: query.operations,
    },
    answerInput: { value: '', maxLength: problem.input_schema.answer.max_length },
    queryJudgement: { state: 'idle' },
    answerJudgement: { state: 'idle' },
    clear: {
      cleared: false,
      clearedCount: run.cleared_problem_ids.length,
      requiredCount: room.required_count,
    },
  }
}

const room = {
  room_id: '1411824c-d357-4941-af76-c76cb827dda6',
  number: 1,
  name: '最初の部屋',
  problem_count: 4,
  required_count: 4,
}
const run = mockContract.getResponseExample(
  'getCurrentRun',
  200,
  'current_run',
) as ActiveRunResponse
const problem = mockContract.getResponseExample(
  'getProblem',
  200,
  'available_problem',
) as ProblemResponse
const query = mockContract.getRequestExample('submitQuery', 'serial_operations') as QueryRequest

export const roomPageFixture = roomViewModelFromFixtures(room, run, problem, query)

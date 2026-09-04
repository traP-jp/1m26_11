import type { MockContract, MockScenarioStep, MockState, MockStateValue } from './contract'

const DEFAULT_STATE: MockState = {
  auth_mode: 'demo',
  problem_authoring_enabled: true,
  image_upload_enabled: true,
  authenticated: true,
  room_exists: true,
  room_is_published: false,
  problem_number_is_available: true,
  room_id_format: 'valid_uuid',
  active_run_exists: false,
  cleared_run_exists: false,
  published_rooms_count: 1,
  problem_exists: true,
  problem_created: false,
  problem_asset_appended: false,
  problem_has_assets: true,
  problem_status: 'available',
  next_problem_status: 'locked',
  query_judgement: 'incorrect',
  answer_judgement: 'incorrect',
  answer_max_length: 50,
  last_required_problem: false,
  run_status: 'active',
}

export interface MockStateStore {
  get(key: string): MockStateValue | undefined
  snapshot(): MockState
  patch(values: MockState): void
  reset(scenarioId?: string): void
  applyStep(scenarioId: string, operationId: string): MockScenarioStep
}

export function createMockState(
  contract: MockContract,
  initialScenarioId?: string,
): MockStateStore {
  let current: MockState = {}

  const reset = (scenarioId?: string) => {
    current = {
      ...DEFAULT_STATE,
      ...(scenarioId ? contract.getScenario(scenarioId).preconditions : {}),
    }
  }

  reset(initialScenarioId)

  return {
    get(key) {
      return current[key]
    },
    snapshot() {
      return { ...current }
    },
    patch(values) {
      Object.assign(current, values)
    },
    reset,
    applyStep(scenarioId, operationId) {
      const step = contract
        .getScenario(scenarioId)
        .steps.find((candidate) => candidate.operationId === operationId)
      if (!step) {
        throw new Error(`scenario ${scenarioId} has no step for ${operationId}`)
      }
      Object.assign(current, step.stateAfter)
      return step
    },
  }
}

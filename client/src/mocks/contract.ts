import { parse } from 'yaml'

export type MockStateValue = boolean | number | string
export type MockState = Record<string, MockStateValue>

export interface MockScenarioResponse {
  status: number
  example?: string
}

export interface MockScenarioStep {
  operationId: string
  requestExample?: string
  response: MockScenarioResponse
  stateAfter: MockState
}

export interface MockScenario {
  id: string
  preconditions: MockState
  steps: MockScenarioStep[]
}

interface MockResponseDefinition {
  examples: Map<string, unknown>
}

interface MockOperation {
  requestExamples: Map<string, unknown>
  responses: Map<number, MockResponseDefinition>
}

export interface MockContract {
  openApiSource: string
  scenarios: MockScenario[]
  getRequestExample(operationId: string, example: string): unknown
  getResponseExample(operationId: string, status: number, example: string): unknown
  getScenario(id: string): MockScenario
}

type UnknownRecord = Record<string, unknown>

const HTTP_METHODS = new Set(['delete', 'get', 'head', 'options', 'patch', 'post', 'put'])

function asRecord(value: unknown, label: string): UnknownRecord {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`)
  }
  return value as UnknownRecord
}

function asString(value: unknown, label: string): string {
  if (typeof value !== 'string') {
    throw new Error(`${label} must be a string`)
  }
  return value
}

function resolveObject(document: UnknownRecord, value: unknown, label: string): UnknownRecord {
  const object = asRecord(value, label)
  const reference = object.$ref
  if (reference === undefined) return object

  const pointer = asString(reference, `${label}.$ref`)
  if (!pointer.startsWith('#/')) {
    throw new Error(`${label} uses unsupported reference ${pointer}`)
  }

  let resolved: unknown = document
  for (const segment of pointer.slice(2).split('/')) {
    resolved = asRecord(resolved, pointer)[segment.replace(/~1/g, '/').replace(/~0/g, '~')]
  }
  return resolveObject(document, resolved, pointer)
}

function getFixture(
  externalValue: unknown,
  fixtureModules: Record<string, unknown>,
  label: string,
): unknown {
  const relativePath = asString(externalValue, `${label}.externalValue`).replace(/^\.\//, '')
  const expectedSuffix = `/openapi/${relativePath}`
  const match = Object.entries(fixtureModules).find(([path]) =>
    path.replace(/\\/g, '/').endsWith(expectedSuffix),
  )
  if (!match) {
    throw new Error(`${label} references missing fixture ${relativePath}`)
  }
  return match[1]
}

function readExamples(
  document: UnknownRecord,
  content: unknown,
  fixtureModules: Record<string, unknown>,
  label: string,
): Map<string, unknown> {
  if (content === undefined) return new Map()
  const mediaTypes = asRecord(content, `${label}.content`)
  const jsonMediaType = mediaTypes['application/json']
  if (jsonMediaType === undefined) return new Map()

  const mediaType = resolveObject(document, jsonMediaType, `${label}.content.application/json`)
  if (mediaType.examples === undefined) return new Map()

  return new Map(
    Object.entries(asRecord(mediaType.examples, `${label}.examples`)).map(([key, value]) => {
      const example = resolveObject(document, value, `${label}.examples.${key}`)
      return [key, getFixture(example.externalValue, fixtureModules, `${label}.examples.${key}`)]
    }),
  )
}

function readOperations(
  document: UnknownRecord,
  fixtureModules: Record<string, unknown>,
): Map<string, MockOperation> {
  const paths = asRecord(document.paths, 'OpenAPI paths')
  const operations = new Map<string, MockOperation>()

  for (const [path, pathValue] of Object.entries(paths)) {
    const pathItem = resolveObject(document, pathValue, `paths.${path}`)
    for (const [method, operationValue] of Object.entries(pathItem)) {
      if (!HTTP_METHODS.has(method)) continue
      const operation = resolveObject(document, operationValue, `${method.toUpperCase()} ${path}`)
      const operationId = asString(
        operation.operationId,
        `${method.toUpperCase()} ${path}.operationId`,
      )

      let requestExamples = new Map<string, unknown>()
      if (operation.requestBody !== undefined) {
        const requestBody = resolveObject(
          document,
          operation.requestBody,
          `${operationId}.requestBody`,
        )
        requestExamples = readExamples(
          document,
          requestBody.content,
          fixtureModules,
          `${operationId}.requestBody`,
        )
      }

      const responses = new Map<number, MockResponseDefinition>()
      for (const [statusText, responseValue] of Object.entries(
        asRecord(operation.responses, `${operationId}.responses`),
      )) {
        const status = Number(statusText)
        if (!Number.isInteger(status)) continue
        const response = resolveObject(
          document,
          responseValue,
          `${operationId}.responses.${statusText}`,
        )
        responses.set(status, {
          examples: readExamples(
            document,
            response.content,
            fixtureModules,
            `${operationId}.responses.${statusText}`,
          ),
        })
      }

      operations.set(operationId, { requestExamples, responses })
    }
  }
  return operations
}

function readState(value: unknown, label: string): MockState {
  if (value === undefined) return {}
  const state: MockState = {}
  for (const [key, entry] of Object.entries(asRecord(value, label))) {
    if (!['boolean', 'number', 'string'].includes(typeof entry)) {
      throw new Error(`${label}.${key} must be a boolean, number, or string`)
    }
    state[key] = entry as MockStateValue
  }
  return state
}

function readScenarios(source: string): MockScenario[] {
  const scenarioDocument = asRecord(parse(source), 'scenario document')
  if (!Array.isArray(scenarioDocument.cases)) {
    throw new Error('scenario document cases must be an array')
  }

  return scenarioDocument.cases.map((caseValue, caseIndex) => {
    const scenario = asRecord(caseValue, `cases[${caseIndex}]`)
    const id = asString(scenario.id, `cases[${caseIndex}].id`)
    if (!Array.isArray(scenario.steps)) {
      throw new Error(`scenario ${id} steps must be an array`)
    }
    const steps = scenario.steps.map((stepValue, stepIndex) => {
      const step = asRecord(stepValue, `scenario ${id} step ${stepIndex}`)
      const response = asRecord(step.response, `scenario ${id} step ${stepIndex}.response`)
      const status = response.status
      if (typeof status !== 'number' || !Number.isInteger(status)) {
        throw new Error(`scenario ${id} step ${stepIndex} response status must be an integer`)
      }
      return {
        operationId: asString(step.operation_id, `scenario ${id} step ${stepIndex}.operation_id`),
        ...(step.request_example === undefined
          ? {}
          : {
              requestExample: asString(
                step.request_example,
                `scenario ${id} step ${stepIndex}.request_example`,
              ),
            }),
        response: {
          status,
          ...(response.example === undefined
            ? {}
            : {
                example: asString(
                  response.example,
                  `scenario ${id} step ${stepIndex}.response.example`,
                ),
              }),
        },
        stateAfter: readState(step.state_after, `scenario ${id} step ${stepIndex}.state_after`),
      }
    })
    return {
      id,
      preconditions: readState(scenario.preconditions, `scenario ${id}.preconditions`),
      steps,
    }
  })
}

function requireOperation(
  operations: Map<string, MockOperation>,
  operationId: string,
): MockOperation {
  const operation = operations.get(operationId)
  if (!operation) throw new Error(`unknown OpenAPI operation ${operationId}`)
  return operation
}

export function createMockContract(
  openApiSource: string,
  scenarioSources: string | readonly string[],
  fixtureModules: Record<string, unknown>,
): MockContract {
  const openApiDocument = asRecord(parse(openApiSource), 'OpenAPI document')
  const operations = readOperations(openApiDocument, fixtureModules)
  const sources = typeof scenarioSources === 'string' ? [scenarioSources] : scenarioSources
  const scenarios = sources.flatMap(readScenarios)

  for (const scenario of scenarios) {
    for (const step of scenario.steps) {
      const operation = requireOperation(operations, step.operationId)
      if (step.requestExample && !operation.requestExamples.has(step.requestExample)) {
        throw new Error(
          `scenario ${scenario.id} references unknown request example ${step.requestExample}`,
        )
      }
      const response = operation.responses.get(step.response.status)
      if (!response) {
        throw new Error(
          `scenario ${scenario.id} references unknown response status ${step.response.status}`,
        )
      }
      if (step.response.example && !response.examples.has(step.response.example)) {
        throw new Error(
          `scenario ${scenario.id} references unknown response example ${step.response.example}`,
        )
      }
    }
  }

  return {
    openApiSource,
    scenarios,
    getRequestExample(operationId, example) {
      const value = requireOperation(operations, operationId).requestExamples.get(example)
      if (value === undefined) {
        throw new Error(`${operationId} has no request example ${example}`)
      }
      return value
    },
    getResponseExample(operationId, status, example) {
      const response = requireOperation(operations, operationId).responses.get(status)
      const value = response?.examples.get(example)
      if (value === undefined) {
        throw new Error(`${operationId} ${status} has no response example ${example}`)
      }
      return value
    },
    getScenario(id) {
      const scenario = scenarios.find((candidate) => candidate.id === id)
      if (!scenario) throw new Error(`unknown mock scenario ${id}`)
      return scenario
    },
  }
}

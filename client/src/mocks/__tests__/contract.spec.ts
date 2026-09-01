import { describe, expect, it } from 'vitest'

import currentRun from '../../../../openapi/examples/runs/active-response.json'
import openApiSource from '../../../../openapi/openapi-v1.yaml?raw'
import scenarioSource from '../../../../openapi/scenarios/p0-cases.yaml?raw'
import { createMockContract } from '../contract'

const fixtureModules = import.meta.glob('../../../../openapi/examples/**/*.json', {
  eager: true,
  import: 'default',
})

describe('OpenAPI mock contract', () => {
  it('resolves scenario response examples from the shared OpenAPI fixtures', () => {
    const contract = createMockContract(openApiSource, scenarioSource, fixtureModules)

    expect(contract.scenarios).toHaveLength(28)
    expect(contract.getResponseExample('getCurrentRun', 200, 'current_run')).toEqual(currentRun)
  })

  it('rejects a scenario that references an unknown response example', () => {
    const invalidScenarioSource = scenarioSource.replace(
      'example: current_run',
      'example: missing_example',
    )

    expect(() => createMockContract(openApiSource, invalidScenarioSource, fixtureModules)).toThrow(
      'missing_example',
    )
  })

  it('keeps the OpenAPI YAML available for the tooling endpoint', () => {
    const contract = createMockContract(openApiSource, scenarioSource, fixtureModules)

    expect(contract.openApiSource).toBe(openApiSource)
  })
})

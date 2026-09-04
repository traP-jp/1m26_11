import { describe, expect, it } from 'vitest'

import currentRun from '../../../../openapi/examples/runs/active-response.json'
import roomActive from '../../../../openapi/examples/rooms/response-active.json'
import openApiSource from '../../../../openapi/openapi-v1.yaml?raw'
import scenarioSource from '../../../../openapi/scenarios/p0-cases.yaml?raw'
import { createMockContract } from '../contract'
import imageUploadScenarioSource from '../../../../openapi/scenarios/image-upload.yaml?raw'
import problemAuthoringScenarioSource from '../../../../openapi/scenarios/problem-authoring.yaml?raw'

const fixtureModules = import.meta.glob('../../../../openapi/examples/**/*.json', {
  eager: true,
  import: 'default',
})

describe('OpenAPI mock contract', () => {
  it('resolves scenario response examples from the shared OpenAPI fixtures', () => {
    const contract = createMockContract(openApiSource, scenarioSource, fixtureModules)

    expect(contract.scenarios).toHaveLength(39)
    expect(contract.getResponseExample('getCurrentRun', 200, 'current_run')).toEqual(currentRun)
    expect(contract.getResponseExample('getRoom', 200, 'active')).toEqual(roomActive)
  })

  it('loads the problem authoring and image upload scenario documents together', () => {
    const contract = createMockContract(
      openApiSource,
      [scenarioSource, imageUploadScenarioSource, problemAuthoringScenarioSource],
      fixtureModules,
    )

    expect(contract.scenarios).toHaveLength(43)
    expect(contract.getScenario('create_string_problem').steps[0]?.operationId).toBe(
      'createProblem',
    )
    expect(contract.getScenario('upload_problem_asset').steps[0]?.operationId).toBe(
      'uploadProblemAsset',
    )
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

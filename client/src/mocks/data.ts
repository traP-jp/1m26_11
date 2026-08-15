import openApiSource from '../../../openapi/openapi-v1.yaml?raw'
import scenarioSource from '../../../openapi/scenarios/p0-cases.yaml?raw'

import { createMockContract } from './contract'

const fixtureModules = import.meta.glob('../../../openapi/examples/**/*.json', {
  eager: true,
  import: 'default',
})

export const mockContract = createMockContract(openApiSource, scenarioSource, fixtureModules)

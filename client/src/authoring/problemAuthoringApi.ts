import type { InjectionKey } from 'vue'

import type { ProblemAuthoringApiClient } from '@/api/client'

export const problemAuthoringApiClientKey: InjectionKey<ProblemAuthoringApiClient> = Symbol(
  'problemAuthoringApiClient',
)

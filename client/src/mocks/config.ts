export interface MswEnvironment {
  dev: boolean
  enabled?: string
}

export function isMswEnabled(environment: MswEnvironment): boolean {
  return environment.dev && environment.enabled !== 'false'
}

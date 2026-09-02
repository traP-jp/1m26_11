export type SerialStatus =
  'unsupported' | 'denied' | 'connecting' | 'connected' | 'disconnected' | 'retry-failed'

export interface SerialStatusNoticeProps {
  status: SerialStatus
  retryLabel?: string
  retryDisabled?: boolean
}

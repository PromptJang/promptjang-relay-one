export type ViewName = 'overview' | 'destinations' | 'mailboxes' | 'events' | 'keys' | 'system'

export interface RevealedSecret {
  value: string
  kind: 'api-key' | 'signing-secret'
}

export interface Destination {
  id: string
  name: string
  url: string
  enabled: boolean
  created_at: string
  updated_at: string
  has_previous_secret: boolean
}

export interface RelayEvent {
  id: string
  destination_id: string
  status: string
  event_type?: string
  correlation_id?: string
  payload: unknown
  content_type: string
  traceparent?: string
  retry_count: number
  max_retries: number
  is_replay: boolean
  source_event_id?: string
  last_error?: string
  created_at: string
  updated_at: string
}

export interface DeliveryAttempt {
  id: string
  status_code?: number
  response_body?: string
  duration_ms: number
  error?: string
  attempted_at: string
}

export interface ApiKey {
  id: string
  name: string
  prefix: string
  last_used_at?: string
  created_at: string
  unrestricted: boolean
  retrievable: boolean
  destination_ids: readonly string[]
}

export interface MailboxSummary {
  name: string
  unread: number
  claimed: number
  acknowledged: number
  created_at: string
}

export interface MailboxMessage {
  id: string
  status: string
  content_type: string
  payload: string
  payload_json?: unknown
  payload_sha256: string
  traceparent?: string
  claim_count: number
  created_at: string
  updated_at: string
}

export interface SystemStatus {
  version: string
  queue: { active: number; retrying: number; delivered: number; expired: number }
  limits: {
    max_payload_bytes: number
    rate_per_destination_per_minute: number
    retention_days: number
    worker_concurrency: number
    delivery_timeout_seconds: number
    retry_delays_seconds: readonly number[]
  }
  telemetry: {
    enabled: boolean
    signals: readonly string[]
    protocol: string
    collector_host?: string
    last_successful_export_at?: string
    last_export_error?: string
  }
}

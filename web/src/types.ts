export type ViewName = 'overview' | 'mailboxes' | 'keys' | 'system'
export interface RevealedSecret { value: string; kind: 'api-key' }
export interface ApiKey { id:string; name:string; prefix:string; last_used_at?:string; created_at:string; retrievable:boolean }
export interface MailboxSummary { name:string; unread:number; claimed:number; acknowledged:number; created_at:string }
export interface MailboxMessage { id:string; status:string; content_type:string; payload:string; payload_json?:unknown; payload_sha256:string; traceparent?:string; claim_count:number; created_at:string; updated_at:string }
export interface SystemStatus {
  version:string; runtime:string; database_path:string; database_bytes:number; mailboxes:number
  messages:{unread:number;claimed:number;acknowledged:number}
  update_check_enabled:boolean
  limits:{max_payload_bytes:number;retention_days:number;max_claim_batch:number}
}
export interface UpdateInfo { enabled:boolean; available:boolean; current_version:string; latest_version?:string; release_url:string; release_notes?:string; checked_at:string; check_error?:string }

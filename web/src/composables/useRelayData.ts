import { readonly, ref, shallowRef } from 'vue'
import type { ApiKey, DeliveryAttempt, Destination, MailboxMessage, MailboxSummary, RelayEvent, RevealedSecret, SystemStatus } from '../types'

type Request = <T>(path: string, init?: RequestInit) => Promise<T>

export function useRelayData(request: Request) {
  const destinations = ref<Destination[]>([])
  const events = ref<RelayEvent[]>([])
  const keys = ref<ApiKey[]>([])
  const mailboxes = ref<MailboxSummary[]>([])
  const mailboxMessages = ref<MailboxMessage[]>([])
  const selectedMailbox = shallowRef('')
  const system = shallowRef<SystemStatus>()
  const selectedEvent = shallowRef<{ event: RelayEvent; attempts: DeliveryAttempt[] }>()
  const loading = shallowRef(false)
  const error = shallowRef('')
  const secret = shallowRef<RevealedSecret>()

  async function run<T>(operation: () => Promise<T>): Promise<T | undefined> {
    error.value = ''
    try { return await operation() }
    catch (cause) { error.value = cause instanceof Error ? cause.message : 'Request failed' }
  }

  async function refresh() {
    loading.value = true
    await run(async () => {
      const [destinationData, eventData, keyData, mailboxData, systemData] = await Promise.all([
        request<{ destinations: Destination[] }>('/api/v1/destinations'),
        request<{ events: RelayEvent[] }>('/api/v1/events'),
        request<{ keys: ApiKey[] }>('/api/v1/keys'),
        request<{ mailboxes: MailboxSummary[] }>('/api/v1/mail'),
        request<SystemStatus>('/api/v1/system'),
      ])
      destinations.value = destinationData.destinations
      events.value = eventData.events
      keys.value = keyData.keys
      mailboxes.value = mailboxData.mailboxes
      system.value = systemData
    })
    loading.value = false
  }

  async function createDestination(input: { name: string; url: string }) {
    const data = await run(() => request<{ secret: string }>('/api/v1/destinations', { method: 'POST', body: JSON.stringify(input) }))
    if (data) { secret.value = { value: data.secret, kind: 'signing-secret' }; await refresh() }
  }
  async function updateDestination(destination: Destination, enabled: boolean) {
    await run(() => request(`/api/v1/destinations/${destination.id}`, { method: 'PATCH', body: JSON.stringify({ name: destination.name, url: destination.url, enabled }) }))
    await refresh()
  }
  async function deleteDestination(id: string) {
    await run(() => request(`/api/v1/destinations/${id}`, { method: 'DELETE' }))
    await refresh()
  }
  async function rotateSecret(id: string) {
    const data = await run(() => request<{ secret: string }>(`/api/v1/destinations/${id}/signing-secret/rotate`, { method: 'POST' }))
    if (data) secret.value = { value: data.secret, kind: 'signing-secret' }
  }
  async function testDestination(id: string) {
    await run(() => request(`/api/v1/destinations/${id}/test`, { method: 'POST' }))
    await refresh()
  }
  async function finishRotation(id: string) {
    await run(() => request(`/api/v1/destinations/${id}/signing-secret/previous`, { method: 'DELETE' }))
    await refresh()
  }
  async function createKey(input: { name: string; destination_ids: string[] }) {
    const data = await run(() => request<{ key: string }>('/api/v1/keys', { method: 'POST', body: JSON.stringify(input) }))
    if (data) { secret.value = { value: data.key, kind: 'api-key' }; await refresh() }
  }
  async function revokeKey(id: string) {
    await run(() => request(`/api/v1/keys/${id}`, { method: 'DELETE' }))
    await refresh()
  }
  async function revealKey(id: string): Promise<string> {
    const data = await request<{ key: string }>(`/api/v1/keys/${id}/secret`)
    return data.key
  }
  async function inspectMailbox(name: string) {
    const data = await run(() => request<{ messages: MailboxMessage[] }>(`/api/v1/mail/${encodeURIComponent(name)}/messages`))
    if (data) {
      selectedMailbox.value = name
      mailboxMessages.value = data.messages
    }
  }
  async function deleteMailbox(name: string) {
    await run(() => request(`/api/v1/mail/${encodeURIComponent(name)}`, { method: 'DELETE' }))
    if (selectedMailbox.value === name) {
      selectedMailbox.value = ''
      mailboxMessages.value = []
    }
    await refresh()
  }
  async function inspectEvent(id: string) {
    const data = await run(() => request<{ event: RelayEvent; attempts: DeliveryAttempt[] }>(`/api/v1/events/${id}`))
    if (data) selectedEvent.value = data
  }
  async function replayEvent(id: string) {
    await run(() => request(`/api/v1/events/${id}/replay`, { method: 'POST' }))
    await refresh()
  }

  return {
    destinations: readonly(destinations), events: readonly(events), keys: readonly(keys), mailboxes: readonly(mailboxes),
    mailboxMessages: readonly(mailboxMessages), selectedMailbox: readonly(selectedMailbox), system: readonly(system),
    selectedEvent: readonly(selectedEvent), loading: readonly(loading), error: readonly(error), secret: readonly(secret),
    refresh, createDestination, updateDestination, deleteDestination, rotateSecret, testDestination, finishRotation, createKey, revokeKey, revealKey,
    inspectMailbox, deleteMailbox,
    inspectEvent, replayEvent, clearSelectedEvent: () => { selectedEvent.value = undefined }, clearSecret: () => { secret.value = undefined },
  }
}

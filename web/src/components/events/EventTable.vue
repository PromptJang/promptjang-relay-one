<script setup lang="ts">
import type { RelayEvent } from '../../types'
import StatusBadge from '../StatusBadge.vue'
defineProps<{ events: readonly RelayEvent[] }>()
const emit = defineEmits<{ inspect: [id: string]; replay: [id: string] }>()
function format(value: string) { return new Date(value).toLocaleString() }
</script>
<template><div class="table-shell"><table><thead><tr><th>Event</th><th>Status</th><th>Retries</th><th>Accepted</th><th></th></tr></thead><tbody><tr v-for="event in events" :key="event.id"><td><button class="event-link" @click="emit('inspect',event.id)"><code>{{ event.id }}</code><small>{{ event.event_type ?? 'untyped' }}</small></button></td><td><StatusBadge :status="event.status" /></td><td>{{ event.retry_count }} / {{ event.max_retries }}</td><td>{{ format(event.created_at) }}</td><td><button class="secondary" @click="emit('replay',event.id)">Replay</button></td></tr></tbody></table><p v-if="!events.length" class="empty">Accepted events will appear here.</p></div></template>
<style scoped>.table-shell{overflow-x:auto;border:1px solid var(--border);border-radius:12px;background:var(--surface)}table{width:100%;min-width:760px;border-collapse:collapse}th,td{padding:13px 15px;border-bottom:1px solid var(--border);text-align:left;font-size:13px}th{color:var(--muted)}.event-link{padding:0;border:0;color:var(--text);background:none;text-align:left}.event-link code,.event-link small{display:block}.event-link small{margin-top:4px;color:var(--muted)}</style>


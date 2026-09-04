<script setup lang="ts">
import { computed } from 'vue'
import { useMessageFilters } from '../../composables/useMessageFilters'
import type { MailboxMessage, MailboxSummary } from '../../types'
import MessageFilters from './MessageFilters.vue'
import StatusBadge from '../StatusBadge.vue'

const props = defineProps<{
  messages: readonly MailboxMessage[]
  selected: string
  summary?: MailboxSummary
}>()
const emit = defineEmits<{ refresh: [name: string] }>()
const messages = computed(() => props.messages)
const filters = useMessageFilters(messages)

function payload(message: MailboxMessage) {
  return message.payload_json === undefined ? message.payload : JSON.stringify(message.payload_json, null, 2)
}
</script>

<template>
  <div class="message-list">
    <div v-if="summary" class="section-heading">
      <div><p class="eyebrow">SELECTED MAILBOX</p><h2>{{ summary.name }}</h2></div>
      <button class="secondary" @click="emit('refresh', summary.name)">Refresh messages</button>
    </div>
    <MessageFilters v-if="selected" v-model:query="filters.query.value" v-model:status="filters.status.value" @clear="filters.clear" />
    <p v-if="selected && filters.filteredMessages.value.length !== messages.length" class="result-count">
      {{ filters.filteredMessages.value.length }} of {{ messages.length }} retained messages
    </p>
    <article v-for="message in filters.filteredMessages.value" :key="message.id" class="panel message-card">
      <div class="message-heading">
        <code>{{ message.id }}</code>
        <StatusBadge :status="message.status" />
      </div>
      <pre><code>{{ payload(message) }}</code></pre>
      <p>{{ message.content_type }} · Claimed {{ message.claim_count }} times · {{ new Date(message.created_at).toLocaleString() }}</p>
    </article>
    <p v-if="selected && !messages.length" class="empty panel">No messages in this mailbox.</p>
    <p v-else-if="selected && !filters.filteredMessages.value.length" class="empty panel">No retained message matches these filters.</p>
    <p v-if="!selected" class="empty panel">Select a mailbox to inspect retained messages.</p>
  </div>
</template>

<style scoped>
.message-list{display:grid;align-content:start;gap:10px}.section-heading{display:flex;justify-content:space-between;align-items:center;gap:12px}.section-heading h2{margin-bottom:0}.result-count{margin:0;color:var(--muted);font-size:12px}.message-heading{display:flex;justify-content:space-between;align-items:center;gap:12px}.message-heading code{overflow:hidden;text-overflow:ellipsis}.message-card{display:grid;gap:12px;padding:18px}.message-card pre{max-height:320px;margin:0;padding:14px;overflow:auto;border:1px solid var(--border);border-radius:9px;background:var(--elevated);white-space:pre-wrap;overflow-wrap:anywhere}.message-card pre code{font-size:12px}.message-card p{margin:0;color:var(--muted);font-size:12px}@media(max-width:520px){.message-heading,.section-heading{align-items:stretch;flex-direction:column}}
</style>

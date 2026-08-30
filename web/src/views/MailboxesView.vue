<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import ConfirmModal from '../components/ConfirmModal.vue'
import StatusBadge from '../components/StatusBadge.vue'
import type { MailboxMessage, MailboxSummary } from '../types'

const props = defineProps<{
  mailboxes: readonly MailboxSummary[]
  messages: readonly MailboxMessage[]
  selected: string
}>()
const emit = defineEmits<{ inspect: [name: string]; remove: [name: string] }>()
const pendingDelete = shallowRef('')
const selectedSummary = computed(() => props.mailboxes.find(mailbox => mailbox.name === props.selected))

function confirmDelete() {
  if (pendingDelete.value) emit('remove', pendingDelete.value)
  pendingDelete.value = ''
}

function payload(message: MailboxMessage) {
  return message.payload_json === undefined ? message.payload : JSON.stringify(message.payload_json, null, 2)
}
</script>

<template>
  <section class="mailbox-layout">
    <div class="mailbox-list">
      <article v-for="mailbox in mailboxes" :key="mailbox.name" class="panel mailbox-card" :class="{ selected: selected === mailbox.name }">
        <div>
          <h3>{{ mailbox.name }}</h3>
          <p>{{ mailbox.unread }} unread · {{ mailbox.claimed }} claimed · {{ mailbox.acknowledged }} acknowledged</p>
        </div>
        <div class="actions">
          <button class="secondary" @click="emit('inspect', mailbox.name)">Inspect</button>
          <button class="danger" @click="pendingDelete = mailbox.name">Delete</button>
        </div>
      </article>
      <p v-if="!mailboxes.length" class="empty panel">No mailboxes yet. A mailbox appears after its first message is accepted.</p>
    </div>

    <div class="message-list">
      <div v-if="selectedSummary" class="section-heading">
        <div><p class="eyebrow">SELECTED MAILBOX</p><h2>{{ selectedSummary.name }}</h2></div>
        <button class="secondary" @click="emit('inspect', selectedSummary.name)">Refresh messages</button>
      </div>
      <article v-for="message in messages" :key="message.id" class="panel message-card">
        <div class="message-heading">
          <code>{{ message.id }}</code>
          <StatusBadge :status="message.status" />
        </div>
        <pre><code>{{ payload(message) }}</code></pre>
        <p>{{ message.content_type }} · Claimed {{ message.claim_count }} times · {{ new Date(message.created_at).toLocaleString() }}</p>
      </article>
      <p v-if="selected && !messages.length" class="empty panel">No messages in this mailbox.</p>
      <p v-if="!selected" class="empty panel">Select a mailbox to inspect retained messages.</p>
    </div>
  </section>

  <ConfirmModal :open="Boolean(pendingDelete)" title="Delete mailbox" message="This permanently deletes the mailbox and every retained message in it. This cannot be undone." confirm-label="Delete mailbox" danger @confirm="confirmDelete" @cancel="pendingDelete = ''" />
</template>

<style scoped>
.mailbox-layout{display:grid;grid-template-columns:minmax(300px,.8fr) minmax(0,1.2fr);gap:18px}.mailbox-list,.message-list{display:grid;align-content:start;gap:10px}.mailbox-card{display:grid;gap:14px;padding:18px}.mailbox-card.selected{border-color:var(--green)}.mailbox-card h3{margin-bottom:5px}.mailbox-card p,.message-card p{margin:0;color:var(--muted);font-size:12px}.actions{display:flex;gap:8px}.message-heading{display:flex;justify-content:space-between;align-items:center;gap:12px}.message-heading code{overflow:hidden;text-overflow:ellipsis}.message-card{display:grid;gap:12px;padding:18px}.message-card pre{max-height:320px;margin:0;padding:14px;overflow:auto;border:1px solid var(--border);border-radius:9px;background:var(--elevated);white-space:pre-wrap;overflow-wrap:anywhere}.message-card pre code{font-size:12px}.section-heading h2{margin-bottom:0}@media(max-width:900px){.mailbox-layout{grid-template-columns:1fr}}@media(max-width:520px){.message-heading,.section-heading{align-items:stretch;flex-direction:column}.actions{justify-content:flex-end}}
</style>

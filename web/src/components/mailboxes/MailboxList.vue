<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import type { MailboxSummary } from '../../types'

const props = defineProps<{ mailboxes: readonly MailboxSummary[]; selected: string }>()
const emit = defineEmits<{ inspect: [name: string]; remove: [name: string] }>()
const query = shallowRef('')
const visibleMailboxes = computed(() => {
  const needle = query.value.trim().toLocaleLowerCase()
  return needle
    ? props.mailboxes.filter((mailbox) => mailbox.name.toLocaleLowerCase().includes(needle))
    : props.mailboxes
})
</script>

<template>
  <div class="mailbox-list">
    <label v-if="mailboxes.length" class="mailbox-search">
      Find mailbox
      <input v-model="query" type="search" placeholder="Mailbox name" />
    </label>
    <article v-for="mailbox in visibleMailboxes" :key="mailbox.name" class="panel mailbox-card" :class="{ selected: selected === mailbox.name }">
      <div>
        <h3>{{ mailbox.name }}</h3>
        <p>{{ mailbox.unread }} unread · {{ mailbox.claimed }} claimed · {{ mailbox.acknowledged }} acknowledged</p>
      </div>
      <div class="actions">
        <button class="secondary" @click="emit('inspect', mailbox.name)">Inspect</button>
        <button class="danger" @click="emit('remove', mailbox.name)">Delete</button>
      </div>
    </article>
    <p v-if="!mailboxes.length" class="empty panel">No mailboxes yet. A mailbox appears after its first message is accepted.</p>
    <p v-else-if="!visibleMailboxes.length" class="empty panel">No mailbox matches “{{ query }}”.</p>
  </div>
</template>

<style scoped>
.mailbox-list{display:grid;align-content:start;gap:10px}.mailbox-search{display:grid;gap:6px;padding:14px;border:1px solid var(--border);border-radius:10px;background:var(--surface);color:var(--muted);font-size:12px;font-weight:650}.mailbox-card{display:grid;gap:14px;padding:18px}.mailbox-card.selected{border-color:var(--green)}.mailbox-card h3{margin-bottom:5px}.mailbox-card p{margin:0;color:var(--muted);font-size:12px}.actions{display:flex;gap:8px}@media(max-width:520px){.actions{justify-content:flex-end}}
</style>

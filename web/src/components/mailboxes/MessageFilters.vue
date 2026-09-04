<script setup lang="ts">
import type { MessageStatusFilter } from '../../composables/useMessageFilters'

const query = defineModel<string>('query', { default: '' })
const status = defineModel<MessageStatusFilter>('status', { default: 'all' })
const emit = defineEmits<{ clear: [] }>()
</script>

<template>
  <div class="filters" role="search" aria-label="Filter mailbox messages">
    <label>
      Search messages
      <input v-model="query" type="search" placeholder="Message ID or payload" />
    </label>
    <label>
      Status
      <select v-model="status">
        <option value="all">All statuses</option>
        <option value="UNREAD">Unread</option>
        <option value="CLAIMED">Claimed</option>
        <option value="ACKNOWLEDGED">Acknowledged</option>
      </select>
    </label>
    <button v-if="query || status !== 'all'" class="secondary" @click="emit('clear')">Clear</button>
  </div>
</template>

<style scoped>
.filters{display:grid;grid-template-columns:minmax(180px,1fr) minmax(150px,.45fr) auto;align-items:end;gap:10px;padding:14px;border:1px solid var(--border);border-radius:10px;background:var(--surface)}
.filters label{display:grid;gap:6px;color:var(--muted);font-size:12px;font-weight:650}.filters input,.filters select{width:100%}.filters button{margin-bottom:1px}@media(max-width:640px){.filters{grid-template-columns:1fr}.filters button{width:100%;margin:0}}
</style>

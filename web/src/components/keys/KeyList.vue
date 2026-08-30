<script setup lang="ts">
import type { ApiKey } from '../../types'

defineProps<{
  keys: readonly ApiKey[]
  copyingId: string | null
  copiedId: string | null
  copyError: string
}>()
const emit = defineEmits<{ copy: [key: ApiKey]; revoke: [id: string] }>()
</script>

<template>
  <div class="cards">
    <p v-if="copyError" class="error" role="alert">{{ copyError }}</p>
    <article v-for="key in keys" :key="key.id" class="key-card">
      <div class="key-details">
        <h3>{{ key.name }}</h3>
        <code>{{ key.prefix }}…</code>
        <p>{{ key.unrestricted ? 'All destinations' : `${key.destination_ids.length} destinations` }} · Last used {{ key.last_used_at ? new Date(key.last_used_at).toLocaleString() : 'never' }}</p>
        <p v-if="!key.retrievable" class="legacy-note">This key predates encrypted retrieval. Replace it to enable full-key copy.</p>
      </div>
      <div class="key-actions">
        <button v-if="key.retrievable" class="secondary" :disabled="copyingId === key.id" :aria-label="`Copy full API key for ${key.name}`" @click="emit('copy', key)">{{ copyingId === key.id ? 'Copying…' : copiedId === key.id ? 'Copied' : 'Copy key' }}</button>
        <button v-else class="secondary" disabled>Copy unavailable</button>
        <button class="danger" @click="emit('revoke', key.id)">Revoke</button>
      </div>
    </article>
    <p v-if="!keys.length" class="empty panel">No producer API keys.</p>
    <p class="copy-status" aria-live="polite">{{ copiedId ? 'Full API key copied to clipboard.' : '' }}</p>
  </div>
</template>

<style scoped>
.cards{display:grid;gap:10px}.key-card{display:flex;justify-content:space-between;align-items:center;gap:16px;padding:18px;border:1px solid var(--border);border-radius:12px;background:var(--surface)}.key-details{min-width:0}.key-card p{margin:7px 0 0;color:var(--muted);font-size:12px}.key-card .legacy-note{color:var(--amber)}.key-actions{display:flex;gap:8px;align-items:center;flex-shrink:0}.copy-status{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}@media(max-width:560px){.key-card{align-items:stretch;flex-direction:column}.key-actions{justify-content:flex-end}}
</style>

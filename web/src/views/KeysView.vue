<script setup lang="ts">
import { shallowRef } from 'vue'
import type { ApiKey, Destination } from '../types'
import ConfirmModal from '../components/ConfirmModal.vue'
import KeyForm from '../components/keys/KeyForm.vue'
import KeyList from '../components/keys/KeyList.vue'

const props = defineProps<{
  keys: readonly ApiKey[]
  destinations: readonly Destination[]
  revealKey: (id: string) => Promise<string>
}>()
const emit = defineEmits<{ create: [input: { name: string; destination_ids: string[] }]; revoke: [id: string] }>()
const pendingRevoke = shallowRef<string | null>(null)
const copyingId = shallowRef<string | null>(null)
const copiedId = shallowRef<string | null>(null)
const copyError = shallowRef('')

function confirmRevoke() {
  if (pendingRevoke.value) emit('revoke', pendingRevoke.value)
  pendingRevoke.value = null
}

async function copyKey(key: ApiKey) {
  copyingId.value = key.id
  copiedId.value = null
  copyError.value = ''
  try {
    const secret = await props.revealKey(key.id)
    await navigator.clipboard.writeText(secret)
    copiedId.value = key.id
  } catch (cause) {
    copyError.value = cause instanceof Error ? cause.message : 'Full API key could not be copied.'
  } finally {
    copyingId.value = null
  }
}
</script>

<template>
  <section class="split">
    <KeyForm :destinations="destinations" @create="emit('create', $event)" />
    <KeyList :keys="keys" :copying-id="copyingId" :copied-id="copiedId" :copy-error="copyError" @copy="copyKey" @revoke="pendingRevoke = $event" />
  </section>
  <ConfirmModal :open="pendingRevoke !== null" title="Revoke API key" message="Producers using this key will stop immediately. This cannot be undone." confirm-label="Revoke key" danger @confirm="confirmRevoke" @cancel="pendingRevoke = null" />
</template>

<style scoped>
.split{display:grid;grid-template-columns:minmax(290px,.7fr) minmax(0,1.3fr);gap:18px}@media(max-width:860px){.split{grid-template-columns:1fr}}
</style>

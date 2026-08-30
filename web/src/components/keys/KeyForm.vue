<script setup lang="ts">
import { reactive } from 'vue'
import type { Destination } from '../../types'

defineProps<{ destinations: readonly Destination[] }>()
const emit = defineEmits<{ create: [input: { name: string; destination_ids: string[] }] }>()
const form = reactive({ name: '', destination_ids: [] as string[] })

function submit() {
  emit('create', { name: form.name, destination_ids: [...form.destination_ids] })
  form.name = ''
  form.destination_ids = []
}
</script>

<template>
  <form class="panel key-form" @submit.prevent="submit">
    <h3>Create API key</h3>
    <label>Name<input v-model="form.name" required maxlength="100" placeholder="Order producer"></label>
    <fieldset>
      <legend>Destination scope</legend>
      <label v-for="destination in destinations" :key="destination.id" class="check"><input v-model="form.destination_ids" type="checkbox" :value="destination.id">{{ destination.name }}</label>
      <p v-if="!destinations.length" class="muted">No destinations exist. An empty scope creates an unrestricted key.</p>
    </fieldset>
    <button>Create API key</button>
    <p class="muted">Keys use the <code>pj_relay_</code> prefix. Full keys are encrypted at rest and remain copyable by the owner. Leaving every destination unchecked grants access to all destinations.</p>
  </form>
</template>

<style scoped>
.key-form{display:grid;gap:16px;align-self:start;padding:20px}.key-form fieldset{display:grid;gap:8px;padding:12px;border:1px solid var(--border);border-radius:9px}.key-form legend{padding:0 6px;color:var(--muted);font-size:12px}.check{display:flex;grid-template-columns:none;align-items:center;gap:8px}.check input{width:auto}
</style>

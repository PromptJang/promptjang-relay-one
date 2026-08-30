<script setup lang="ts">
import type { Destination, RelayEvent, SystemStatus, ViewName } from '../types'
import StatusBadge from '../components/StatusBadge.vue'
defineProps<{ system?: SystemStatus; destinations: readonly Destination[]; events: readonly RelayEvent[] }>()
const emit = defineEmits<{ navigate: [view: ViewName] }>()
</script>

<template>
  <section class="metrics"><article><span>Active queue</span><strong>{{ system?.queue.active ?? 0 }}</strong></article><article><span>Delivered</span><strong>{{ system?.queue.delivered ?? 0 }}</strong></article><article><span>Expired</span><strong>{{ system?.queue.expired ?? 0 }}</strong></article><article><span>Destinations</span><strong>{{ destinations.length }}</strong></article></section>
  <section class="grid"><article class="panel"><div class="section-heading"><div><p class="eyebrow">SETUP</p><h3>First delivery</h3></div></div><ol class="checklist"><li :class="{ done: destinations.length }">Create a destination</li><li>Generate a scoped API key</li><li :class="{ done: events.length }">Send and inspect an event</li></ol><button class="secondary" @click="emit('navigate', destinations.length ? 'events' : 'destinations')">Continue setup</button></article><article class="panel"><div class="section-heading"><div><p class="eyebrow">RECENT</p><h3>Delivery activity</h3></div><button class="link" @click="emit('navigate','events')">View all</button></div><ul class="activity"><li v-for="event in events.slice(0,5)" :key="event.id"><div><code>{{ event.id.slice(0,12) }}</code><small>{{ event.event_type ?? 'untyped' }}</small></div><StatusBadge :status="event.status" /></li><li v-if="!events.length" class="empty">No accepted events yet.</li></ul></article></section>
</template>

<style scoped>
.metrics{display:grid;grid-template-columns:repeat(4,1fr);gap:14px;margin-bottom:18px}.metrics article{padding:18px;border:1px solid var(--border);border-radius:12px;background:var(--surface)}.metrics span{color:var(--muted);font-size:13px}.metrics strong{display:block;margin-top:10px;font:600 28px var(--font-mono)}.grid{display:grid;grid-template-columns:1fr 1.4fr;gap:18px}.panel{padding:20px}.checklist{display:grid;gap:13px;padding:0;list-style:none}.checklist li{color:var(--muted)}.checklist li::before{display:inline-grid;width:20px;height:20px;margin-right:9px;border:1px solid var(--border);border-radius:50%;place-items:center;content:""}.checklist li.done{color:var(--text)}.checklist li.done::before{border-color:var(--green);color:var(--green);content:"✓"}.activity{padding:0;margin:0;list-style:none}.activity li{display:flex;justify-content:space-between;align-items:center;padding:12px 0;border-bottom:1px solid var(--border)}.activity code,.activity small{display:block}.activity small{margin-top:4px;color:var(--muted)}@media(max-width:900px){.metrics{grid-template-columns:1fr 1fr}.grid{grid-template-columns:1fr}}@media(max-width:520px){.metrics{grid-template-columns:1fr}}
</style>

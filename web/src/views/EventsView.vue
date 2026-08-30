<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import type { DeliveryAttempt, RelayEvent } from '../types'
import EventDetail from '../components/events/EventDetail.vue'
import EventTable from '../components/events/EventTable.vue'
const props = defineProps<{ events: readonly RelayEvent[]; selected?: { event: RelayEvent; attempts: readonly DeliveryAttempt[] } }>()
const emit = defineEmits<{ inspect: [id: string]; replay: [id: string]; close: [] }>()
const status = shallowRef('')
const query = shallowRef('')
const visible = computed(() => props.events.filter(event => (!status.value || event.status === status.value) && (!query.value || `${event.id} ${event.event_type ?? ''}`.toLowerCase().includes(query.value.toLowerCase()))))
</script>
<template><EventDetail v-if="selected" :event="selected.event" :attempts="selected.attempts" @close="emit('close')" @replay="emit('replay',$event)" /><div class="filters"><input v-model="query" type="search" placeholder="Search ID or event type" aria-label="Search events"><select v-model="status" aria-label="Filter status"><option value="">All statuses</option><option v-for="value in ['QUEUED','PROCESSING','RETRYING','DELIVERED','EXPIRED']" :key="value">{{ value }}</option></select></div><EventTable :events="visible" @inspect="emit('inspect',$event)" @replay="emit('replay',$event)" /></template>
<style scoped>.filters{display:flex;gap:10px;margin-bottom:14px}.filters input{max-width:360px}.filters select{padding:10px;border:1px solid var(--border);border-radius:8px;color:var(--text);background:var(--elevated)}@media(max-width:560px){.filters{flex-direction:column}.filters input{max-width:none}}</style>

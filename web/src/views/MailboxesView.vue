<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import ConfirmModal from '../components/ConfirmModal.vue'
import MailboxList from '../components/mailboxes/MailboxList.vue'
import MessageList from '../components/mailboxes/MessageList.vue'
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

</script>

<template>
  <section class="mailbox-layout">
    <MailboxList :mailboxes="mailboxes" :selected="selected" @inspect="emit('inspect', $event)" @remove="pendingDelete = $event" />
    <MessageList :messages="messages" :selected="selected" :summary="selectedSummary" @refresh="emit('inspect', $event)" />
  </section>

  <ConfirmModal :open="Boolean(pendingDelete)" title="Delete mailbox" message="This permanently deletes the mailbox and every retained message in it. This cannot be undone." confirm-label="Delete mailbox" danger @confirm="confirmDelete" @cancel="pendingDelete = ''" />
</template>

<style scoped>
.mailbox-layout{display:grid;grid-template-columns:minmax(300px,.8fr) minmax(0,1.2fr);gap:18px}@media(max-width:900px){.mailbox-layout{grid-template-columns:1fr}}
</style>

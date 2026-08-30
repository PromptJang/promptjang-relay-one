<script setup lang="ts">
import { onMounted, shallowRef } from 'vue'
import AppShell from './components/AppShell.vue'
import SecretNotice from './components/SecretNotice.vue'
import { useRelayApi } from './composables/useRelayApi'
import { useTheme } from './composables/useTheme'
import { useRelayData } from './composables/useRelayData'
import DestinationsView from './views/DestinationsView.vue'
import EventsView from './views/EventsView.vue'
import KeysView from './views/KeysView.vue'
import MailboxesView from './views/MailboxesView.vue'
import OverviewView from './views/OverviewView.vue'
import SystemView from './views/SystemView.vue'
import type { ViewName } from './types'

const view = shallowRef<ViewName>('overview')
const { request } = useRelayApi()
const relay = useRelayData(request)
const { theme, set: setTheme } = useTheme()

onMounted(() => { void relay.refresh() })
</script>

<template>
  <AppShell :view="view" :version="relay.system.value?.version" :theme="theme" @set-theme="setTheme" @navigate="view=$event" @refresh="relay.refresh">
    <p v-if="relay.error.value" class="error banner" role="alert">{{ relay.error.value }}</p>
    <SecretNotice v-if="relay.secret.value" :secret="relay.secret.value" @dismiss="relay.clearSecret" />
    <OverviewView v-if="view==='overview'" :system="relay.system.value" :destinations="relay.destinations.value" :events="relay.events.value" @navigate="view=$event" />
    <DestinationsView v-else-if="view==='destinations'" :destinations="relay.destinations.value" @create="relay.createDestination" @toggle="relay.updateDestination" @test="relay.testDestination" @rotate="relay.rotateSecret" @finish-rotation="relay.finishRotation" @remove="relay.deleteDestination" />
    <MailboxesView v-else-if="view==='mailboxes'" :mailboxes="relay.mailboxes.value" :messages="relay.mailboxMessages.value" :selected="relay.selectedMailbox.value" @inspect="relay.inspectMailbox" @remove="relay.deleteMailbox" />
    <EventsView v-else-if="view==='events'" :events="relay.events.value" :selected="relay.selectedEvent.value" @inspect="relay.inspectEvent" @replay="relay.replayEvent" @close="relay.clearSelectedEvent" />
    <KeysView v-else-if="view==='keys'" :keys="relay.keys.value" :destinations="relay.destinations.value" :reveal-key="relay.revealKey" @create="relay.createKey" @revoke="relay.revokeKey" />
    <SystemView v-else :system="relay.system.value" />
  </AppShell>
</template>

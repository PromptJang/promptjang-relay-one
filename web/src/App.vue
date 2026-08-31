<script setup lang="ts">
import { onMounted, shallowRef } from 'vue'
import AppShell from './components/AppShell.vue'
import SecretNotice from './components/SecretNotice.vue'
import UpdatePrompt from './components/UpdatePrompt.vue'
import { useRelayApi } from './composables/useRelayApi'
import { useRelayData } from './composables/useRelayData'
import { useTheme } from './composables/useTheme'
import KeysView from './views/KeysView.vue'
import IntegrationsView from './views/IntegrationsView.vue'
import MailboxesView from './views/MailboxesView.vue'
import OverviewView from './views/OverviewView.vue'
import SystemView from './views/SystemView.vue'
import type { ViewName } from './types'

const view=shallowRef<ViewName>('overview')
const {request}=useRelayApi(), relay=useRelayData(request), {theme,set:setTheme}=useTheme()
onMounted(()=>{void relay.refresh();void relay.checkUpdate()})
</script>

<template>
  <AppShell :view="view" :version="relay.system.value?.version" :surface="relay.system.value?.surface" :theme="theme" @set-theme="setTheme" @navigate="view=$event" @refresh="relay.refresh" @open-docs="relay.openDocs">
    <UpdatePrompt v-if="relay.update.value?.available" :update="relay.update.value" @open-update="relay.openRelease" />
    <p v-if="relay.error.value" class="error banner" role="alert">{{ relay.error.value }}</p>
    <SecretNotice v-if="relay.secret.value" :secret="relay.secret.value" @dismiss="relay.clearSecret" />
    <OverviewView v-if="view==='overview'" :system="relay.system.value" :mailboxes="relay.mailboxes.value" @navigate="view=$event" />
    <MailboxesView v-else-if="view==='mailboxes'" :mailboxes="relay.mailboxes.value" :messages="relay.mailboxMessages.value" :selected="relay.selectedMailbox.value" @inspect="relay.inspectMailbox" @remove="relay.deleteMailbox" />
    <KeysView v-else-if="view==='keys'" :keys="relay.keys.value" :reveal-key="relay.revealKey" @create="relay.createKey" @revoke="relay.revokeKey" />
    <IntegrationsView v-else-if="view==='integrations'" :keys="relay.keys.value" />
    <SystemView v-else :system="relay.system.value" :update="relay.update.value" @check-update="relay.checkUpdate(true)" @open-update="relay.openRelease" />
  </AppShell>
</template>

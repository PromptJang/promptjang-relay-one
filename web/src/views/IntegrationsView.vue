<script setup lang="ts">
import { computed, onMounted, shallowRef } from 'vue'
import McpClientCard from '../components/integrations/McpClientCard.vue'
import { useMcpSetup } from '../composables/useMcpSetup'
import { useRelayApi } from '../composables/useRelayApi'
import type { ApiKey, McpClientId } from '../types'

const props=defineProps<{keys:readonly ApiKey[]}>()
const {request}=useRelayApi()
const setup=useMcpSetup(request)
const keyId=shallowRef('')
const copied=shallowRef(false)
const hasKeys=computed(()=>props.keys.length>0)

async function install(client:McpClientId){ if(keyId.value) await setup.install(client,keyId.value) }
async function copySkill(){
  const command=setup.status.value?.skill_install_command
  if(!command)return
  await navigator.clipboard.writeText(command);copied.value=true
  window.setTimeout(()=>{copied.value=false},1600)
}
onMounted(()=>{keyId.value=props.keys[0]?.id??'';void setup.load()})
</script>

<template>
  <section class="layout">
    <article class="panel setup-panel">
      <p class="eyebrow">LOCAL MCP</p>
      <h2>Connect a CLI agent</h2>
      <p class="muted">Choose an existing API key. Relay One writes a local stdio MCP entry with the absolute path of this running app.</p>
      <label>API key<select v-model="keyId" :disabled="!hasKeys"><option v-for="key in keys" :key="key.id" :value="key.id">{{ key.name }} · {{ key.prefix }}…</option></select></label>
      <p v-if="!hasKeys" class="warning">Create an API key first.</p>
      <p class="privacy">The selected key is stored in the client’s local MCP configuration. No mailbox is fixed; the agent chooses a mailbox on every tool call.</p>
      <p v-if="setup.error.value" class="error banner" role="alert">{{ setup.error.value }}</p>
      <p v-if="setup.result.value" class="success" role="status">Installed for {{ setup.result.value.client }}. {{ setup.result.value.verification }}</p>
      <div v-if="setup.status.value" class="clients">
        <McpClientCard v-for="client in setup.status.value.clients" :key="client.id" :client="client" :pending="setup.pendingClient.value===client.id" :disabled="!keyId || Boolean(setup.pendingClient.value)" @install="install" />
      </div>
    </article>
    <article class="panel skill-panel">
      <p class="eyebrow">AGENT SKILL</p>
      <h2>Teach the agent the workflow</h2>
      <p class="muted">MCP provides tools. The PromptJang skill teaches agents how to push, claim, acknowledge, and nack messages safely.</p>
      <pre>{{ setup.status.value?.skill_install_command??'Loading command…' }}</pre>
      <button class="secondary" :disabled="!setup.status.value" @click="copySkill">{{ copied?'Copied':'Copy install command' }}</button>
      <a href="https://github.com/PromptJang/promptjang-relay-skill" target="_blank" rel="noopener">View skill repository ↗</a>
    </article>
  </section>
</template>

<style scoped>
.layout{display:grid;grid-template-columns:minmax(0,1.7fr) minmax(280px,1fr);gap:18px}.setup-panel,.skill-panel{padding:24px}.setup-panel>label{max-width:460px;margin:22px 0 12px}.clients{display:grid;gap:10px;margin-top:22px}.privacy,.warning,.success{padding:11px 12px;border-radius:8px;font-size:13px}.privacy{color:var(--muted);background:var(--elevated)}.warning{color:var(--amber)}.success{color:var(--green);background:color-mix(in srgb,var(--green) 8%,var(--surface))}.skill-panel{align-self:start}.skill-panel pre{padding:14px;overflow-wrap:anywhere;white-space:pre-wrap;border:1px solid var(--border);border-radius:8px;background:var(--elevated);font-size:12px}.skill-panel button{width:100%;margin-bottom:16px}.skill-panel a{color:var(--green)}@media(max-width:900px){.layout{grid-template-columns:1fr}}
</style>

<script setup lang="ts">
import { computed } from 'vue'
import type { McpClientStatus } from '../../types'

const props = defineProps<{ client: McpClientStatus; selectedKeyId: string; pending: boolean }>()
const emit = defineEmits<{ diagnose: [] }>()
const configured = computed(() => props.client.configured && props.client.key_id === props.selectedKeyId)
const verified = computed(() => configured.value && Boolean(props.client.adapter_verified_at))
const active = computed(() => configured.value && Boolean(props.client.last_activity_at))
</script>

<template>
  <div class="diagnostics" aria-label="MCP connection status">
    <ol>
      <li :class="{ complete: configured }"><span>{{ configured ? '✓' : '1' }}</span><div><strong>Configured</strong><small v-if="client.configured && !configured">Uses another API key</small></div></li>
      <li :class="{ complete: verified }"><span>{{ verified ? '✓' : '2' }}</span><div><strong>Adapter verified</strong><small v-if="client.adapter_verified_at && configured">{{ new Date(client.adapter_verified_at).toLocaleString() }}</small></div></li>
      <li :class="{ complete: active }"><span>{{ active ? '✓' : '3' }}</span><div><strong>Agent activity detected</strong><small v-if="client.last_activity_at && configured">{{ new Date(client.last_activity_at).toLocaleString() }}</small></div></li>
    </ol>
    <button class="secondary" :disabled="!configured || pending" @click="emit('diagnose')">{{ pending ? 'Checking…' : 'Run MCP check' }}</button>
  </div>
</template>

<style scoped>
.diagnostics{display:grid;grid-template-columns:minmax(0,1fr) auto;align-items:center;gap:16px;padding-top:14px;border-top:1px solid var(--border)}ol{display:flex;flex-wrap:wrap;gap:12px 20px;padding:0;margin:0;list-style:none}li{display:flex;align-items:center;gap:8px;color:var(--muted);font-size:12px}li>span{display:grid;width:22px;height:22px;place-items:center;border:1px solid var(--border);border-radius:50%;font:650 11px var(--font-mono)}li div{display:grid;gap:2px}li small{color:var(--amber);font-size:10px}li.complete{color:var(--text)}li.complete>span{border-color:var(--green);background:color-mix(in srgb,var(--green) 14%,transparent);color:var(--green)}li.complete small{color:var(--muted)}@media(max-width:700px){.diagnostics{grid-template-columns:1fr}.diagnostics button{width:100%}}
</style>

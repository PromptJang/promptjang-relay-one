<script setup lang="ts">
import { computed } from 'vue'
import type { McpClientStatus } from '../../types'
import McpDiagnostics from './McpDiagnostics.vue'
const props=defineProps<{client:McpClientStatus;selectedKeyId:string;pending:boolean;diagnosing:boolean;disabled:boolean}>()
const emit=defineEmits<{install:[client:McpClientStatus['id']];diagnose:[client:McpClientStatus['id']]}>()
const configuredForSelectedKey=computed(()=>props.client.configured&&props.client.key_id===props.selectedKeyId)
</script>

<template>
  <article class="client-card">
    <div class="client-heading">
      <div>
        <h3>{{ client.label }}</h3>
        <p v-if="client.available" class="available">Available locally</p>
        <p v-else class="missing">Client command not found</p>
        <code v-if="client.command_path">{{ client.command_path }}</code>
      </div>
      <button :disabled="disabled || !client.available" @click="emit('install',client.id)">{{ pending?'Installing…':configuredForSelectedKey?'Reinstall MCP':'Install MCP' }}</button>
    </div>
    <McpDiagnostics :client="client" :selected-key-id="selectedKeyId" :pending="diagnosing" @diagnose="emit('diagnose',client.id)" />
  </article>
</template>

<style scoped>
.client-card{display:grid;gap:16px;padding:18px;border:1px solid var(--border);border-radius:10px;background:var(--elevated)}.client-heading{display:flex;align-items:center;justify-content:space-between;gap:20px}.client-card h3,.client-card p{margin:0}.client-card code{display:block;max-width:560px;margin-top:7px;overflow:hidden;color:var(--muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}.available,.missing{margin-top:6px!important;font-size:12px}.available{color:var(--green)}.missing{color:var(--amber)}@media(max-width:560px){.client-heading{align-items:stretch;flex-direction:column}.client-heading>button{width:100%}}
</style>

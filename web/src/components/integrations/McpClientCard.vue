<script setup lang="ts">
import type { McpClientStatus } from '../../types'
defineProps<{client:McpClientStatus;pending:boolean;disabled:boolean}>()
const emit=defineEmits<{install:[client:McpClientStatus['id']]}>()
</script>

<template>
  <article class="client-card">
    <div>
      <h3>{{ client.label }}</h3>
      <p v-if="client.available" class="available">Available locally</p>
      <p v-else class="missing">Client command not found</p>
      <code v-if="client.command_path">{{ client.command_path }}</code>
    </div>
    <button :disabled="disabled || !client.available" @click="emit('install',client.id)">{{ pending?'Installing…':'Install MCP' }}</button>
  </article>
</template>

<style scoped>
.client-card{display:flex;align-items:center;justify-content:space-between;gap:20px;padding:18px;border:1px solid var(--border);border-radius:10px;background:var(--elevated)}.client-card h3,.client-card p{margin:0}.client-card code{display:block;max-width:560px;margin-top:7px;overflow:hidden;color:var(--muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}.available,.missing{margin-top:6px!important;font-size:12px}.available{color:var(--green)}.missing{color:var(--amber)}@media(max-width:560px){.client-card{align-items:stretch;flex-direction:column}.client-card button{width:100%}}
</style>

<script setup lang="ts">
import { shallowRef } from 'vue'
import type { UpdateInfo } from '../types'
const props=defineProps<{update:UpdateInfo}>()
const emit=defineEmits<{openUpdate:[]}>()
const dismissed=shallowRef(localStorage.getItem('pj_one_dismissed_update')===props.update.latest_version)
function later(){if(props.update.latest_version)localStorage.setItem('pj_one_dismissed_update',props.update.latest_version);dismissed.value=true}
</script>
<template><aside v-if="!dismissed" class="update" role="status"><div><strong>Relay One {{ update.latest_version }} is available</strong><p>{{ update.release_notes?.split('\n').find(line=>line.trim()) || 'A new stable release is ready.' }}</p></div><div class="actions"><button @click="emit('openUpdate')">Get update</button><button class="secondary" @click="later">Not now</button></div></aside></template>
<style scoped>.update{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:14px 16px;margin-bottom:18px;border:1px solid color-mix(in srgb,var(--green) 45%,var(--border));border-radius:12px;background:color-mix(in srgb,var(--green) 8%,var(--surface))}.update p{margin:5px 0 0;color:var(--muted);font-size:13px}.actions{display:flex;gap:8px}@media(max-width:640px){.update{align-items:stretch;flex-direction:column}}</style>

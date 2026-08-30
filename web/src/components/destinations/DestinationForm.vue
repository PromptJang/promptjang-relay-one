<script setup lang="ts">
import { reactive } from 'vue'
const emit = defineEmits<{ create: [input: { name: string; url: string }] }>()
const form = reactive({ name: '', url: '' })
function submit() { emit('create', { ...form }); form.name = ''; form.url = '' }
</script>
<template><form class="panel" @submit.prevent="submit"><h3>Create destination</h3><label>Name<input v-model="form.name" required maxlength="100" placeholder="Order service"></label><label>HTTPS URL<input v-model="form.url" type="url" required placeholder="https://service.example/webhooks"></label><button>Create destination</button><p class="muted">Private addresses require an explicit operator CIDR allowlist.</p></form></template>
<style scoped>.panel{display:grid;gap:16px;align-self:start;padding:20px}.panel p{margin:0;font-size:12px}</style>


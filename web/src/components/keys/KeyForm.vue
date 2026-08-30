<script setup lang="ts">
import { reactive } from 'vue'
const emit=defineEmits<{create:[input:{name:string}]}>()
const form=reactive({name:''})
function submit(){emit('create',{name:form.name});form.name=''}
</script>
<template><form class="panel form" @submit.prevent="submit"><h3>Create agent API key</h3><label>Name<input v-model="form.name" required maxlength="100" placeholder="Codex inbox agent"></label><button>Create API key</button><p class="muted">Keys use the <code>pj_one_</code> prefix and can access local mailboxes. Full keys are encrypted at rest and remain copyable.</p></form></template>
<style scoped>.form{display:grid;gap:16px;align-self:start;padding:20px}</style>

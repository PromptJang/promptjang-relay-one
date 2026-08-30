<script setup lang="ts">
import { reactive, shallowRef } from 'vue'
const emit = defineEmits<{ login: [username: string, password: string] }>()
defineProps<{ error: string; loading: boolean }>()
const form = reactive({ username: '', password: '' })
const submitted = shallowRef(false)
function submit() { submitted.value = true; emit('login', form.username, form.password) }
</script>

<template>
  <main class="login-shell"><section class="login-card"><p class="eyebrow">RELAY · SELF-HOSTED</p><h1>PromptJang <span>Relay</span></h1><p class="muted">Durable delivery for webhooks and agents, on your PostgreSQL.</p><form @submit.prevent="submit"><label>Username<input v-model="form.username" type="text" required autocomplete="username"></label><label>Password<input v-model="form.password" type="password" required autocomplete="current-password"></label><p v-if="error && submitted" class="error" role="alert">{{ error }}</p><button :disabled="loading">{{ loading ? 'Signing in…' : 'Sign in' }}</button></form><a class="docs-link" href="/docs">Read public documentation →</a></section></main>
</template>

<style scoped>
.login-shell{min-height:100vh;display:grid;place-items:center;padding:24px}.login-card{width:min(430px,100%);padding:34px;border:1px solid var(--border);border-radius:18px;background:var(--surface);box-shadow:0 30px 80px rgb(0 0 0/.24)}.login-card h1 span{color:var(--green)}.login-card form{display:grid;gap:16px;margin-top:28px}.docs-link{display:inline-block;margin-top:22px;color:var(--muted);font-size:13px;text-decoration:none}.docs-link:hover,.docs-link:focus-visible{color:var(--green)}.docs-link:focus-visible{outline:2px solid var(--green);outline-offset:3px;border-radius:3px}
</style>

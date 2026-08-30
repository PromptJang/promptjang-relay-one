<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import type { RevealedSecret } from '../types'
const props = defineProps<{ secret: RevealedSecret }>()
const emit = defineEmits<{ dismiss: [] }>()
const copyState = shallowRef<'idle' | 'copied' | 'failed'>('idle')
const guidance = computed(() => props.secret.kind === 'api-key'
  ? 'It is encrypted at rest and remains copyable from API keys. Use it as a Bearer token for event and mailbox requests.'
  : 'Use a Standard Webhooks v1 verifier at the receiver.')
async function copy() {
  try {
    await navigator.clipboard.writeText(props.secret.value)
    copyState.value = 'copied'
  } catch {
    copyState.value = 'failed'
  }
}
</script>

<template>
  <section class="secret-notice" role="status">
    <div><strong>{{ secret.kind === 'api-key' ? 'API key created' : 'Copy this signing secret now' }}</strong><p><template v-if="secret.kind === 'signing-secret'">It will not be shown again. </template>{{ guidance }}</p></div>
    <code>{{ secret.value }}</code><button @click="copy">{{ copyState === 'copied' ? 'Copied' : 'Copy' }}</button><button class="secondary" @click="emit('dismiss')">Dismiss</button>
    <a v-if="secret.kind === 'signing-secret'" href="https://github.com/standard-webhooks/standard-webhooks" target="_blank" rel="noreferrer">Verification guide</a>
    <span class="copy-status" aria-live="polite">{{ copyState === 'copied' ? 'Secret copied to clipboard.' : copyState === 'failed' ? 'Clipboard access failed. Select and copy the secret manually.' : '' }}</span>
  </section>
</template>

<style scoped>
.secret-notice{display:flex;align-items:center;gap:12px;padding:14px;margin-bottom:20px;border:1px solid color-mix(in srgb,var(--green) 42%,transparent);border-radius:12px;background:color-mix(in srgb,var(--green) 8%,var(--surface))}.secret-notice p{margin:3px 0 0;color:var(--muted);font-size:12px}.secret-notice code{min-width:0;overflow:hidden;text-overflow:ellipsis}.copy-status{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}@media(max-width:760px){.secret-notice{align-items:stretch;flex-direction:column}}
</style>

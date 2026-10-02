<script setup lang="ts">
import type { UpdateInfo } from '../types'
import { onMounted, onUnmounted, useTemplateRef } from 'vue'
import { useVerifiedUpdate } from '../composables/useVerifiedUpdate'
defineProps<{ update: UpdateInfo }>()
const emit = defineEmits<{ close: []; manual: [] }>()
const { installing, error, desktop, install } = useVerifiedUpdate()
const dialog = useTemplateRef<HTMLElement>('dialog')
const previousFocus = document.activeElement as HTMLElement | null
onMounted(() => dialog.value?.querySelector<HTMLButtonElement>('button')?.focus())
onUnmounted(() => previousFocus?.focus())
function keyboard(event: KeyboardEvent) {
  if (event.key === 'Escape' && !installing.value) emit('close')
  if (event.key !== 'Tab') return
  const buttons = Array.from(dialog.value?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') || [])
  if (!buttons.length) { event.preventDefault(); return }
  const first = buttons[0], last = buttons[buttons.length - 1]
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus() }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus() }
}
</script>

<template>
  <div class="backdrop">
    <section ref="dialog" role="dialog" aria-modal="true" aria-labelledby="update-title" class="panel update-dialog" @keydown="keyboard">
      <h2 id="update-title">Update to {{ update.latest_version }}?</h2>
      <p>The app will verify the signed download, save a recovery backup, and restart.
        Mailbox operations pause during startup verification. Failed startup restores the previous app and database.</p>
      <pre class="notes">{{ update.release_notes || 'No release notes provided.' }}</pre>
      <p v-if="error" role="alert" class="error">{{ error }}</p>
      <p v-if="installing" role="status">Downloading and verifying. Keep the app open until it restarts.</p>
      <div class="actions">
        <button v-if="desktop && update.latest_version" :disabled="installing" autofocus
          @click="install(update.latest_version)">Update and restart</button>
        <button v-else @click="emit('manual')">Open release download</button>
        <button class="secondary" :disabled="installing" @click="emit('close')">Not now</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.backdrop{position:fixed;inset:0;z-index:50;display:grid;place-items:center;padding:20px;background:#0009}
.update-dialog{width:min(620px,100%);max-height:85vh;overflow:auto;padding:24px}
.notes{white-space:pre-wrap;overflow-wrap:anywhere;max-height:35vh;overflow:auto}
.actions{display:flex;flex-wrap:wrap;gap:10px;margin-top:20px}
</style>

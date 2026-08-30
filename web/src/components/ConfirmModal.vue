<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

const props = defineProps<{ open: boolean; title: string; message: string; confirmLabel?: string; danger?: boolean }>()
const emit = defineEmits<{ confirm: []; cancel: [] }>()
const confirmButton = ref<HTMLButtonElement | null>(null)

watch(() => props.open, async (open) => {
  if (open) {
    document.addEventListener('keydown', onKeydown, true)
    await nextTick()
    confirmButton.value?.focus()
  } else {
    document.removeEventListener('keydown', onKeydown, true)
  }
})

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('cancel')
}

function onOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) emit('cancel')
}
</script>

<template>
  <div v-if="open" class="overlay" @click="onOverlayClick">
    <div class="modal" role="dialog" aria-modal="true" :aria-label="title">
      <h3>{{ title }}</h3>
      <p>{{ message }}</p>
      <div class="actions">
        <button class="secondary" @click="emit('cancel')">Cancel</button>
        <button ref="confirmButton" :class="danger ? 'danger' : ''" @click="emit('confirm')">{{ confirmLabel ?? 'Confirm' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay{position:fixed;inset:0;z-index:60;display:flex;align-items:center;justify-content:center;background:rgba(9,12,14,.62);padding:20px}
.modal{width:100%;max-width:420px;padding:24px;border:1px solid var(--border);border-radius:14px;background:var(--surface);box-shadow:0 24px 60px rgba(0,0,0,.35)}
.modal h3{margin:0 0 10px;font-size:17px}
.modal p{margin:0 0 22px;color:var(--muted);line-height:1.55}
.actions{display:flex;justify-content:flex-end;gap:10px}
</style>

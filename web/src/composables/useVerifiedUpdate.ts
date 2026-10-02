import { readonly, shallowRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useVerifiedUpdate() {
  const installing = shallowRef(false)
  const error = shallowRef('')
  const desktop = '__TAURI_INTERNALS__' in window
  async function install(version: string) {
    if (!desktop || installing.value) return
    installing.value = true
    error.value = ''
    try { await invoke('install_update', { version }) }
    catch (cause) { error.value = cause instanceof Error ? cause.message : String(cause) }
    finally { installing.value = false }
  }
  return { installing: readonly(installing), error: readonly(error), desktop, install }
}

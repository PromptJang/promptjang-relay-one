import { invoke } from '@tauri-apps/api/core'

export type DesktopLinkCommand = 'open_docs' | 'open_release' | 'open_skill'

export function useExternalLinks() {
  async function openExternal(url: string, desktopCommand: DesktopLinkCommand) {
    if ('__TAURI_INTERNALS__' in window) {
      await invoke(desktopCommand)
      return
    }

    window.open(url, '_blank', 'noopener,noreferrer')
  }

  return { openExternal }
}

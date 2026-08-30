import { ref } from 'vue'

export type ThemeChoice = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'relay-theme'

function preferred(): ThemeChoice {
  const stored = localStorage.getItem(STORAGE_KEY)
  return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system'
}

function apply(choice: ThemeChoice) {
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const dark = choice === 'dark' || (choice === 'system' && systemDark)
  document.documentElement.dataset.theme = dark ? 'dark' : 'light'
}

const theme = ref<ThemeChoice>(preferred())

export function useTheme() {
  function set(next: ThemeChoice) {
    theme.value = next
    if (next === 'system') localStorage.removeItem(STORAGE_KEY)
    else localStorage.setItem(STORAGE_KEY, next)
    apply(next)
  }

  const media = window.matchMedia('(prefers-color-scheme: dark)')
  media.addEventListener('change', () => {
    if (theme.value === 'system') apply('system')
  })

  apply(theme.value)

  return { theme, set }
}

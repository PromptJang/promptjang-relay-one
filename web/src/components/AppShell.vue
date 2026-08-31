<script setup lang="ts">
import { onMounted, onUnmounted, shallowRef } from 'vue'
import type { ThemeChoice } from '../composables/useTheme'
import type { ViewName } from '../types'

defineProps<{
  view: ViewName
  version?: string
  surface?: 'desktop' | 'browser'
  theme: ThemeChoice
}>()

const emit = defineEmits<{
  navigate: [view: ViewName]
  refresh: []
  openDocs: []
  setTheme: [theme: ThemeChoice]
}>()

const items: { id: ViewName; label: string }[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'mailboxes', label: 'Mailboxes' },
  { id: 'keys', label: 'API keys' },
  { id: 'integrations', label: 'Integrations' },
  { id: 'system', label: 'System' },
]

const drawerOpen = shallowRef(false)

function navigate(view: ViewName) {
  drawerOpen.value = false
  emit('navigate', view)
}

function openDocs() {
  drawerOpen.value = false
  emit('openDocs')
}

function closeOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') drawerOpen.value = false
}

onMounted(() => window.addEventListener('keydown', closeOnEscape))
onUnmounted(() => window.removeEventListener('keydown', closeOnEscape))
</script>

<template>
  <div class="app-shell">
    <header class="mobile-bar">
      <div class="mobile-brand">
        <p class="eyebrow">RELAY ONE · LOCAL</p>
        <strong>PromptJang <span>Relay One</span></strong>
      </div>
      <button
        class="menu-button secondary"
        aria-controls="primary-navigation"
        :aria-expanded="drawerOpen"
        aria-label="Open navigation"
        @click="drawerOpen = true"
      >
        <span aria-hidden="true">☰</span>
      </button>
    </header>

    <button
      v-if="drawerOpen"
      class="drawer-backdrop"
      aria-label="Close navigation"
      @click="drawerOpen = false"
    />

    <aside id="primary-navigation" class="sidebar" :class="{ open: drawerOpen }">
      <div class="sidebar-heading">
        <div>
          <p class="eyebrow">RELAY ONE · LOCAL</p>
          <h2>PromptJang <span>Relay One</span></h2>
        </div>
        <button class="drawer-close secondary" aria-label="Close navigation" @click="drawerOpen = false">×</button>
      </div>

      <nav aria-label="Primary">
        <button
          v-for="item in items"
          :key="item.id"
          :class="{ active: view === item.id }"
          @click="navigate(item.id)"
        >
          {{ item.label }}
        </button>
        <button class="nav-link" @click="openDocs">
          Documentation <span aria-hidden="true">↗</span>
        </button>
      </nav>

      <div class="sidebar-status">
        <span class="connected" />
        {{ surface === 'desktop' ? 'Desktop app' : 'Local server' }} · {{ version ? `v${version}` : 'version loading' }}
      </div>
    </aside>

    <main class="content">
      <header class="page-header">
        <div>
          <p class="eyebrow">LOCAL WORKSPACE</p>
          <h1>{{ items.find(item => item.id === view)?.label }}</h1>
        </div>
        <div class="header-actions">
          <div class="theme-switch" role="radiogroup" aria-label="Color theme">
            <button
              v-for="choice in ['system', 'light', 'dark'] as ThemeChoice[]"
              :key="choice"
              :class="{ active: theme === choice }"
              :aria-pressed="theme === choice"
              :title="`Theme: ${choice}`"
              @click="emit('setTheme', choice)"
            >
              {{ choice === 'system' ? 'Auto' : choice === 'light' ? 'Light' : 'Dark' }}
            </button>
          </div>
          <button class="secondary" @click="emit('refresh')">Refresh</button>
        </div>
      </header>
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
}
.mobile-bar,
.drawer-close,
.drawer-backdrop {
  display: none;
}
.sidebar {
  position: sticky;
  top: 0;
  z-index: 20;
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
  border-right: 1px solid var(--border);
  background: var(--surface);
}
.sidebar-heading h2 span,
.mobile-brand span {
  color: var(--green);
}
.sidebar-heading h2 {
  margin-bottom: 0;
}
.sidebar nav {
  display: grid;
  gap: 5px;
  margin-top: 30px;
}
.sidebar nav button {
  min-height: 40px;
  padding: 9px 14px;
  border: 1px solid transparent;
  border-radius: 8px;
  color: var(--muted);
  background: transparent;
  text-align: left;
  font-size: inherit;
  font-weight: 650;
}
.sidebar .nav-link {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.sidebar .nav-link:hover,
.sidebar .nav-link:focus-visible,
.sidebar nav button:hover {
  color: var(--green);
  transform: none;
}
.sidebar nav button.active {
  color: var(--green);
  background: color-mix(in srgb, var(--green) 9%, transparent);
}
.sidebar-status {
  margin-top: auto;
  padding: 10px;
  color: var(--muted);
  font-size: 11px;
}
.sidebar-status span {
  display: inline-block;
  width: 7px;
  height: 7px;
  margin-right: 6px;
  border-radius: 50%;
  background: var(--neutral);
}
.sidebar-status span.connected {
  background: var(--green);
}
.content {
  width: 100%;
  max-width: 1300px;
  min-width: 0;
  padding: 32px;
  margin: auto;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  margin-bottom: 24px;
}
.page-header h1 {
  margin-bottom: 0;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}
.theme-switch {
  display: flex;
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}
.theme-switch button {
  min-height: 30px;
  padding: 4px 10px;
  border: none;
  border-radius: 0;
  background: transparent;
  color: var(--muted);
  font-size: 12px;
  font-weight: 600;
}
.theme-switch button.active {
  color: #041410;
  background: var(--green);
}

@media (max-width: 1024px) {
  .app-shell {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto 1fr;
  }
  .mobile-bar {
    position: sticky;
    top: 0;
    z-index: 15;
    min-height: 70px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 94%, transparent);
    backdrop-filter: blur(12px);
  }
  .mobile-brand .eyebrow {
    margin-bottom: 4px;
  }
  .mobile-brand strong {
    font-size: 18px;
  }
  .menu-button {
    min-width: 42px;
    padding-inline: 11px;
    font-size: 20px;
    line-height: 1;
  }
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: block;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    border-radius: 0;
    background: rgb(0 0 0 / 55%);
    cursor: default;
  }
  .drawer-backdrop:hover {
    transform: none;
  }
  .sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    z-index: 40;
    width: min(320px, calc(100vw - 48px));
    height: 100dvh;
    padding: 22px 18px;
    box-shadow: 16px 0 48px rgb(0 0 0 / 35%);
    transform: translateX(-105%);
    visibility: hidden;
    transition: transform 180ms ease, visibility 180ms;
  }
  .sidebar.open {
    transform: translateX(0);
    visibility: visible;
  }
  .sidebar-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .drawer-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    padding: 4px 10px;
    font-size: 26px;
    line-height: 1;
  }
  .content {
    grid-row: 2;
    max-width: none;
    padding: 28px 24px;
  }
}

@media (max-width: 640px) {
  .mobile-bar {
    padding-inline: 14px;
  }
  .content {
    padding: 22px 14px;
  }
  .page-header {
    align-items: stretch;
    flex-direction: column;
    gap: 14px;
  }
  .header-actions {
    justify-content: space-between;
  }
}
</style>

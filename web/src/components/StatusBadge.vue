<script setup lang="ts">
defineProps<{ status: string }>()
</script>

<template><span class="status-badge" :data-state="status.toLowerCase()">{{ status }}</span></template>

<style scoped>
.status-badge{display:inline-flex;align-items:center;padding:4px 8px;border-radius:999px;color:var(--blue);background:color-mix(in srgb,var(--blue) 12%,transparent);font:600 11px var(--font-mono)}
.status-badge::before{width:6px;height:6px;margin-right:6px;border-radius:50%;background:currentColor;content:""}
.status-badge[data-state="delivered"],.status-badge[data-state="enabled"],.status-badge[data-state="acknowledged"]{color:var(--green)}
.status-badge[data-state="retrying"],.status-badge[data-state="processing"],.status-badge[data-state="claimed"]{color:var(--amber)}
.status-badge[data-state="expired"],.status-badge[data-state="disabled"]{color:var(--red)}
</style>

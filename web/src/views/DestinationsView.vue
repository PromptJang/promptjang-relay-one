<script setup lang="ts">
import { shallowRef } from 'vue'
import type { Destination } from '../types'
import DestinationForm from '../components/destinations/DestinationForm.vue'
import DestinationList from '../components/destinations/DestinationList.vue'
import ConfirmModal from '../components/ConfirmModal.vue'
defineProps<{ destinations: readonly Destination[] }>()
const emit = defineEmits<{ create: [input: { name: string; url: string }]; toggle: [destination: Destination, enabled: boolean]; test: [id: string]; rotate: [id: string]; finishRotation: [id: string]; remove: [id: string] }>()
const pending = shallowRef<{ action: 'remove' | 'rotate'; id: string } | null>(null)
function requestRemove(id: string) { pending.value = { action: 'remove', id } }
function requestRotate(id: string) { pending.value = { action: 'rotate', id } }
function confirmPending() {
  if (!pending.value) return
  if (pending.value.action === 'remove') emit('remove', pending.value.id)
  else emit('rotate', pending.value.id)
  pending.value = null
}
</script><template><section class="split"><DestinationForm @create="emit('create',$event)" /><DestinationList :destinations="destinations" @toggle="(destination,enabled)=>emit('toggle',destination,enabled)" @test="emit('test',$event)" @rotate="requestRotate" @finish-rotation="emit('finishRotation',$event)" @remove="requestRemove" /></section><ConfirmModal :open="pending !== null" :title="pending?.action === 'remove' ? 'Delete destination' : 'Rotate signing secret'" :message="pending?.action === 'remove' ? 'Event history will be retained. New events are rejected while the destination is deleted.' : 'Both v1 signatures stay in one webhook-signature header until you finish rotation. Store the new secret before updating the receiver.'" :confirm-label="pending?.action === 'remove' ? 'Delete' : 'Rotate secret'" :danger="pending?.action === 'remove'" @confirm="confirmPending" @cancel="pending = null" /></template>
<style scoped>.split{display:grid;grid-template-columns:minmax(280px,.65fr) minmax(0,1.35fr);gap:18px}@media(max-width:860px){.split{grid-template-columns:1fr}}</style>

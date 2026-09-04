import { computed, shallowRef, toValue, type MaybeRefOrGetter } from 'vue'
import type { MailboxMessage } from '../types'

export type MessageStatusFilter = 'all' | 'UNREAD' | 'CLAIMED' | 'ACKNOWLEDGED'

export function useMessageFilters(messages: MaybeRefOrGetter<readonly MailboxMessage[]>) {
  const query = shallowRef('')
  const status = shallowRef<MessageStatusFilter>('all')

  const filteredMessages = computed(() => {
    const needle = query.value.trim().toLocaleLowerCase()
    return toValue(messages).filter((message) => {
      if (status.value !== 'all' && message.status !== status.value) return false
      if (!needle) return true
      return `${message.id}\n${message.payload}`.toLocaleLowerCase().includes(needle)
    })
  })

  function clear() {
    query.value = ''
    status.value = 'all'
  }

  return { query, status, filteredMessages, clear }
}

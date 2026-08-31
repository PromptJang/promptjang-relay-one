import { readonly, ref, shallowRef } from 'vue'
import type { ApiKey, MailboxMessage, MailboxSummary, RevealedSecret, SystemStatus, UpdateInfo } from '../types'
import { useExternalLinks } from './useExternalLinks'
type Request = <T>(path:string, init?:RequestInit)=>Promise<T>

export function useRelayData(request:Request) {
  const {openExternal}=useExternalLinks()
  const keys=ref<ApiKey[]>([]), mailboxes=ref<MailboxSummary[]>([]), mailboxMessages=ref<MailboxMessage[]>([])
  const selectedMailbox=shallowRef(''), system=shallowRef<SystemStatus>(), update=shallowRef<UpdateInfo>()
  const loading=shallowRef(false), error=shallowRef(''), secret=shallowRef<RevealedSecret>()
  async function run<T>(operation:()=>Promise<T>):Promise<T|undefined>{ error.value=''; try{return await operation()}catch(cause){error.value=cause instanceof Error?cause.message:typeof cause==='string'?cause:'Request failed'} }
  async function refresh(){ loading.value=true; await run(async()=>{const [k,m,s]=await Promise.all([request<{keys:ApiKey[]}>('/api/v1/keys'),request<{mailboxes:MailboxSummary[]}>('/api/v1/mail'),request<SystemStatus>('/api/v1/system')]);keys.value=k.keys;mailboxes.value=m.mailboxes;system.value=s});loading.value=false }
  async function checkUpdate(refresh=false){const data=await run(()=>request<UpdateInfo>(`/api/v1/update${refresh?'?refresh=true':''}`));if(data)update.value=data}
  async function createKey(input:{name:string}){const data=await run(()=>request<{key:string}>('/api/v1/keys',{method:'POST',body:JSON.stringify(input)}));if(data){secret.value={value:data.key,kind:'api-key'};await refresh()}}
  async function revokeKey(id:string){await run(()=>request(`/api/v1/keys/${id}`,{method:'DELETE'}));await refresh()}
  async function revealKey(id:string){return (await request<{key:string}>(`/api/v1/keys/${id}/secret`)).key}
  async function inspectMailbox(name:string){const data=await run(()=>request<{messages:MailboxMessage[]}>(`/api/v1/mail/${encodeURIComponent(name)}/messages`));if(data){selectedMailbox.value=name;mailboxMessages.value=data.messages}}
  async function deleteMailbox(name:string){await run(()=>request(`/api/v1/mail/${encodeURIComponent(name)}`,{method:'DELETE'}));if(selectedMailbox.value===name){selectedMailbox.value='';mailboxMessages.value=[]}await refresh()}
  async function openDocs(){await run(()=>openExternal(new URL('/docs',window.location.origin).toString(),'open_docs'))}
  async function openRelease(){await run(()=>openExternal('https://github.com/PromptJang/promptjang-relay-one/releases/latest','open_release'))}
  return {keys:readonly(keys),mailboxes:readonly(mailboxes),mailboxMessages:readonly(mailboxMessages),selectedMailbox:readonly(selectedMailbox),system:readonly(system),update:readonly(update),loading:readonly(loading),error:readonly(error),secret:readonly(secret),refresh,checkUpdate,createKey,revokeKey,revealKey,inspectMailbox,deleteMailbox,openDocs,openRelease,clearSecret:()=>{secret.value=undefined}}
}

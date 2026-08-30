import { readonly, shallowRef } from 'vue'
import type { McpClientId, McpInstallResult, McpStatus } from '../types'

type Request = <T>(path:string, init?:RequestInit)=>Promise<T>

export function useMcpSetup(request:Request) {
  const status=shallowRef<McpStatus>()
  const pendingClient=shallowRef<McpClientId>()
  const error=shallowRef('')
  const result=shallowRef<McpInstallResult>()

  async function load(){
    error.value=''
    try{ status.value=await request<McpStatus>('/api/v1/integrations/mcp') }
    catch(cause){ error.value=cause instanceof Error?cause.message:'Could not load MCP setup' }
  }
  async function install(client:McpClientId,keyId:string){
    pendingClient.value=client;error.value='';result.value=undefined
    try{
      result.value=await request<McpInstallResult>('/api/v1/integrations/mcp',{method:'POST',body:JSON.stringify({client,key_id:keyId})})
    }catch(cause){ error.value=cause instanceof Error?cause.message:'MCP setup failed' }
    finally{ pendingClient.value=undefined }
  }
  return {status:readonly(status),pendingClient:readonly(pendingClient),error:readonly(error),result:readonly(result),load,install}
}

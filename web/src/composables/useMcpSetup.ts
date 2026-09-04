import { readonly, shallowRef } from 'vue'
import type { McpClientId, McpDiagnosticResult, McpInstallResult, McpStatus } from '../types'

type Request = <T>(path:string, init?:RequestInit)=>Promise<T>

export function useMcpSetup(request:Request) {
  const status=shallowRef<McpStatus>()
  const pendingClient=shallowRef<McpClientId>()
  const pendingDiagnostic=shallowRef<McpClientId>()
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
      await load()
    }catch(cause){ error.value=cause instanceof Error?cause.message:'MCP setup failed' }
    finally{ pendingClient.value=undefined }
  }
  async function diagnose(client:McpClientId,keyId:string){
    pendingDiagnostic.value=client;error.value='';result.value=undefined
    try{
      const diagnostic=await request<McpDiagnosticResult>('/api/v1/integrations/mcp/diagnose',{method:'POST',body:JSON.stringify({client,key_id:keyId})})
      result.value={installed:true,client:diagnostic.client,server_name:'promptjang',verification:diagnostic.message}
      await load()
    }catch(cause){ error.value=cause instanceof Error?cause.message:'MCP diagnostic failed' }
    finally{ pendingDiagnostic.value=undefined }
  }
  return {status:readonly(status),pendingClient:readonly(pendingClient),pendingDiagnostic:readonly(pendingDiagnostic),error:readonly(error),result:readonly(result),load,install,diagnose}
}

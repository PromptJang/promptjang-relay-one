import { spawn, spawnSync } from 'node:child_process'
import { mkdtemp, rm, writeFile, unlink, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createInterface } from 'node:readline'
import assert from 'node:assert/strict'
import net from 'node:net'

const directory = await mkdtemp(join(tmpdir(), 'pj-one-envelope-'))
const socket = net.createServer()
await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve))
const port = socket.address().port
await new Promise(resolve => socket.close(resolve))
const base = 'http://127.0.0.1:' + port
const binary = resolve('target/debug/promptjang-relay-one')
let server, mcp
const wait = ms => new Promise(resolve => setTimeout(resolve, ms))
async function start() {
  server = spawn(binary, ['--data-dir', directory, '--port', String(port), 'serve', '--no-open'], { stdio: 'ignore' })
  for (let i = 0; i < 80; i++) {
    try { if ((await fetch(base + '/ready')).ok) return } catch {}
    await wait(100)
  }
  throw new Error('Server did not become ready')
}
async function stop() {
  if (!server || server.exitCode !== null) return
  const exited = new Promise(resolve => server.once('exit', resolve))
  server.kill('SIGINT')
  await exited
}
try {
  await start()
  const keyResponse = await fetch(base + '/api/v1/keys', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name: 'temporary-uat' }),
  })
  assert.equal(keyResponse.status, 201)
  const { key } = await keyResponse.json()
  const keyId = (await (await fetch(base + '/api/v1/keys')).json()).keys[0].id
  assert.equal((await (await fetch(base + '/api/v1/keys/' + keyId + '/secret')).json()).key, key)
  mcp = spawn(binary, ['mcp'], { env: { ...process.env, PJ_ONE_URL: base, PJ_ONE_API_KEY: key }, stdio: ['pipe', 'pipe', 'inherit'] })
  const lines = createInterface({ input: mcp.stdout })[Symbol.asyncIterator]()
  let sequence = 0
  async function rpc(method, params = {}) {
    mcp.stdin.write(JSON.stringify({ jsonrpc: '2.0', id: ++sequence, method, params }) + '\n')
    const line = await lines.next()
    assert(!line.done)
    return JSON.parse(line.value).result
  }
  assert((await rpc('initialize')).capabilities.tools)
  const tools = (await rpc('tools/list')).tools
  assert.equal(tools.length, 5)
  assert(tools.every(tool => tool.outputSchema?.type === 'object'))
  assert(tools[0].inputSchema.$defs.agent_message.required.includes('schema'))
  const call = (name, args) => rpc('tools/call', { name, arguments: args })
  const task = { schema: 'promptjang.agent-message.v1', kind: 'task', correlation_id: 'uat', task: 'Review', reply_to: 'results' }
  const pushed = await call('mail_push', { mailbox: 'shared', payload: task, idempotency_key: 'task-one' })
  assert(!pushed.isError)
  assert.equal(JSON.parse(pushed.content[0].text).id, pushed.structuredContent.id)
  const duplicate = await call('mail_push', { mailbox: 'shared', payload: task, idempotency_key: 'task-one' })
  assert.equal(duplicate.structuredContent.id, pushed.structuredContent.id)
  assert.equal((await call('mail_push', { mailbox: 'shared', payload: { ...task, task: 'Different' }, idempotency_key: 'task-one' })).isError, true)
  const invalid = await call('mail_push', { mailbox: 'shared', payload: { schema: task.schema, kind: 'result' } })
  assert.equal(invalid.isError, true)
  assert.equal(invalid.structuredContent, undefined)
  await stop()
  await start()
  const claimed = await call('mail_claim', { mailbox: 'shared', limit: 1 })
  const message = claimed.structuredContent.messages[0]
  assert.deepEqual(message.payload_json, task)
  const result = { schema: task.schema, kind: 'result', correlation_id: 'uat', in_reply_to: message.id, status: 'succeeded', summary: 'Done' }
  assert(!(await call('mail_push', { mailbox: 'results', payload: result, idempotency_key: 'result:' + message.id })).isError)
  assert.equal((await call('mail_ack', { mailbox: 'shared', id: message.id, claim_token: message.claim_token })).structuredContent.status, 'ACKNOWLEDGED')
  const resultClaim = await call('mail_claim', { mailbox: 'results', limit: 1 })
  assert.deepEqual(resultClaim.structuredContent.messages[0].payload_json, result)
  const resultMessage = resultClaim.structuredContent.messages[0]
  assert.equal((await call('mail_nack', { mailbox: 'results', id: resultMessage.id, claim_token: resultMessage.claim_token })).structuredContent.status, 'UNREAD')
  assert(!(await call('mail_push', { mailbox: 'legacy', payload: 'plain text' })).isError)
  await writeFile(join(directory, 'update-pending.json'), '{}')
  assert.equal((await fetch(base + '/api/v1/keys')).status, 503)
  assert.equal((await fetch(base + '/ready')).status, 200)
  await unlink(join(directory, 'update-pending.json'))
  assert.equal((await fetch(base + '/docs/agent-envelope')).status, 200)
  assert.equal((await fetch(base + '/docs/verified-updates')).status, 200)
  await stop()
  const archive = join(directory, 'mailbox-export.json')
  const exported = spawnSync(binary, ['--data-dir', directory, 'export', '--output', archive], { encoding: 'utf8' })
  assert.equal(exported.status, 0, exported.stderr)
  assert((await readFile(archive, 'utf8')).includes(pushed.structuredContent.id))
  const importedDirectory = join(directory, 'imported')
  const imported = spawnSync(binary, ['--data-dir', importedDirectory, 'import', '--input', archive], { encoding: 'utf8' })
  assert.equal(imported.status, 0, imported.stderr)
  console.log('PASS: real stdio MCP task/result, output schemas, deduplication, restart persistence, ack, legacy payload, and update probation')
} finally {
  mcp?.stdin.end()
  await stop()
  await rm(directory, { recursive: true, force: true })
}

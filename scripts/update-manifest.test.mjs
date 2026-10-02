import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtemp, rm, writeFile, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'

test('manifest requires every target and records checksums/signatures', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'pj-one-manifest-'))
  try {
    const env = { ...process.env, GITHUB_REF_NAME: 'v0.4.0' }
    const run = () => spawnSync(process.execPath, ['scripts/update-manifest.mjs', directory], { env, encoding: 'utf8' })
    assert.notEqual(run().status, 0)
    for (const [target, suffix] of [
      ['aarch64-apple-darwin', '.app.tar.gz'],
      ['x86_64-apple-darwin', '.app.tar.gz'],
      ['x86_64-pc-windows-msvc', '.exe'],
      ['x86_64-unknown-linux-gnu', '.AppImage'],
      ['aarch64-unknown-linux-gnu', '.AppImage'],
    ]) {
      const artifact = join(directory, target + '--relay' + suffix)
      await writeFile(artifact, 'fixture artifact')
      await writeFile(artifact + '.sig', 'fixture signature')
    }
    const result = run()
    assert.equal(result.status, 0, result.stderr)
    const manifest = JSON.parse(await readFile(join(directory, 'latest.json'), 'utf8'))
    assert.equal(manifest.repository, 'PromptJang/promptjang-relay-one')
    assert.equal(manifest.version, '0.4.0')
    assert.equal(Object.keys(manifest.platforms).length, 5)
    for (const artifact of Object.values(manifest.platforms)) {
      assert.match(artifact.sha256, /^[a-f0-9]{64}$/)
      assert.equal(artifact.signature, 'fixture signature')
      assert(artifact.url.startsWith('https://github.com/PromptJang/promptjang-relay-one/releases/download/v0.4.0/'))
    }
    // This tests manifest generation, not cryptographic validity of fixture signatures.
  } finally { await rm(directory, { recursive: true, force: true }) }
})

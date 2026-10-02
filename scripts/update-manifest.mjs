import { readdir, readFile, writeFile } from 'node:fs/promises'
import { resolve, join } from 'node:path'
import { createHash } from 'node:crypto'

const directory = resolve(process.argv[2] || 'release-assets')
const version = process.env.GITHUB_REF_NAME
if (!/^v\d+\.\d+\.\d+$/.test(version || '')) throw new Error('Stable release tag required')
const targets = {
  'aarch64-apple-darwin': 'darwin-aarch64',
  'x86_64-apple-darwin': 'darwin-x86_64',
  'x86_64-pc-windows-msvc': 'windows-x86_64',
  'x86_64-unknown-linux-gnu': 'linux-x86_64',
  'aarch64-unknown-linux-gnu': 'linux-aarch64',
}
const files = await readdir(directory)
const platforms = {}
for (const [target, platform] of Object.entries(targets)) {
  const file = files.find(name => name.startsWith(target + '--') &&
    (name.endsWith('.app.tar.gz') || name.endsWith('.AppImage') || name.endsWith('.exe')))
  if (!file) throw new Error('Missing updater artifact for ' + target)
  const signature = (await readFile(join(directory, file + '.sig'), 'utf8')).trim()
  if (!signature) throw new Error('Missing signature: ' + file)
  platforms[platform] = {
    signature,
    sha256: createHash('sha256').update(await readFile(join(directory, file))).digest('hex'),
    url: 'https://github.com/PromptJang/promptjang-relay-one/releases/download/' +
      version + '/' + encodeURIComponent(file),
  }
}
await writeFile(join(directory, 'latest.json'), JSON.stringify({
  version: version.slice(1),
  repository: 'PromptJang/promptjang-relay-one',
  workflow: '.github/workflows/release.yml',
  tag: version,
  notes: await readFile('RELEASE_NOTES.md', 'utf8'),
  pub_date: new Date().toISOString(),
  platforms,
}, null, 2) + '\n')

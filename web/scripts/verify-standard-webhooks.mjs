import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { Webhook } from 'standardwebhooks'

const fixture = JSON.parse(
  await readFile(resolve(import.meta.dirname, '../../tests/fixtures/standard-webhooks-v1.json'), 'utf8'),
)
const webhook = new Webhook(fixture.secret)

const signed = webhook.sign(fixture.event_id, new Date(fixture.timestamp * 1000), fixture.payload)
if (signed !== fixture.signature) {
  throw new Error(`signature mismatch: expected ${fixture.signature}, received ${signed}`)
}

const liveTimestamp = Math.floor(Date.now() / 1000)
const liveSignature = webhook.sign(fixture.event_id, new Date(liveTimestamp * 1000), fixture.payload)

webhook.verify(fixture.payload, {
  'webhook-id': fixture.event_id,
  'webhook-timestamp': String(liveTimestamp),
  'webhook-signature': liveSignature,
})

console.log('Standard Webhooks compatibility fixture verified')

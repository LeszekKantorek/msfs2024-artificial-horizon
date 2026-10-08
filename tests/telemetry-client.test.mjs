import test from 'node:test';
import assert from 'node:assert/strict';
import { createTelemetryClient, validateSnapshot } from '../web/telemetry-client.js';
import { readFileSync } from 'node:fs';

const live = (sequence = 1, age = 0) => ({ schema_version: 1, sequence,
  source: 'demo', state: 'live', sample_age_ms: age,
  attitude: { pitch_deg: 5, roll_deg: 15 } });

function harness() {
  let time = 0;
  let nextId = 0;
  const timers = new Map();
  const connections = [];
  const statuses = [];
  const client = createTelemetryClient({ onStatus: value => statuses.push(value),
    now: () => time,
    schedule: (fn, ms) => { const id = ++nextId; timers.set(id, { fn, at: time + ms }); return id; },
    cancel: id => timers.delete(id),
    makeEventSource: url => {
      assert.equal(url, '/api/v1/events');
      const connection = { closed: false, close() { this.closed = true; },
        addEventListener(name, listener) { assert.equal(name, 'telemetry'); this.receive = listener; },
        send(value) { this.receive({ data: JSON.stringify(value) }); } };
      connections.push(connection);
      return connection;
    },
  });
  return { client, connections, statuses, timers,
    get status() { return statuses.at(-1); },
    advance(ms) {
      time += ms;
      for (;;) {
        const entry = [...timers.entries()].find(([, timer]) => timer.at <= time);
        if (!entry) break;
        timers.delete(entry[0]); entry[1].fn();
      }
    },
  };
}

test('validator accepts independent fixtures and rejects malformed/unsupported telemetry', () => {
  const fixtures = JSON.parse(readFileSync(new URL('./fixtures/snapshots.json', import.meta.url)));
  for (const fixture of fixtures) assert.ok(validateSnapshot(fixture));
  for (const invalid of [null, {}, { ...live(), schema_version: 2 },
    { ...live(), state: 'unknown' }, { ...live(), source: 'other' },
    { ...live(), sequence: 1.5 }, { ...live(), sequence: Number.MAX_SAFE_INTEGER + 1 },
    { ...live(), sample_age_ms: -1 }, { ...live(), sample_age_ms: null },
    { ...live(), attitude: { pitch_deg: 91, roll_deg: 0 } },
    { ...live(), attitude: { pitch_deg: 0, roll_deg: 180 } },
    { ...live(), attitude: { pitch_deg: NaN, roll_deg: 0 } },
    { ...live(), state: 'paused' }]) assert.ok(!validateSnapshot(invalid));
});

test('one subscription retries after two seconds and accepts sequence restart', () => {
  const h = harness();
  h.client.start(); h.client.start();
  assert.equal(h.connections.length, 1);
  const first = h.connections[0];
  first.onopen(); first.send(live(50));
  first.onerror(); first.onerror();
  assert.ok(first.closed);
  assert.equal(h.status.transport, 'reconnecting');
  h.advance(1999); assert.equal(h.connections.length, 1);
  h.advance(1); assert.equal(h.connections.length, 2);
  first.send(live(100)); assert.equal(h.status.state, 'waiting');
  const second = h.connections[1];
  second.onopen(); second.send(live(1));
  assert.equal(h.status.state, 'live');
  assert.equal(h.status.source, 'demo');
  second.onerror(); h.client.stop(); h.advance(2000);
  assert.equal(h.connections.length, 2);
  assert.equal(h.timers.size, 0);
});

test('freshness accounts for server age; duplicates and open events cannot refresh it', () => {
  const h = harness(); h.client.start();
  const connection = h.connections[0];
  connection.onopen(); connection.send(live(10, 500));
  h.advance(499); assert.equal(h.status.state, 'live');
  connection.send(live(10, 0));
  h.advance(1); assert.equal(h.status.state, 'stale');
  assert.equal(h.connections.length, 1);
  connection.send(live(11, 1000)); assert.equal(h.status.state, 'stale');
  connection.send(live(12)); assert.equal(h.status.state, 'live');
});

test('invalid messages and explicit unavailable states never appear live', () => {
  const h = harness(); h.client.start();
  const connection = h.connections[0]; connection.onopen(); connection.send(live());
  connection.receive({ data: 'broken JSON' }); assert.equal(h.status.state, 'invalid');
  h.advance(2000); assert.equal(h.status.state, 'invalid');
  for (const [index, state] of ['paused', 'waiting', 'disconnected', 'invalid', 'stale'].entries()) {
    connection.send({ ...live(index + 2), state, attitude: null });
    assert.equal(h.status.state, state);
  }
  connection.send(live(10)); assert.equal(h.status.state, 'live');
});

test('suspend releases connection and resume waits for a fresh snapshot', () => {
  const h = harness(); h.client.start(); h.connections[0].onopen(); h.connections[0].send(live());
  h.client.stop(); assert.ok(h.connections[0].closed); assert.equal(h.timers.size, 0);
  h.client.start(); assert.equal(h.status.state, 'waiting');
  assert.equal(h.connections.length, 2);
  h.connections[1].onopen(); assert.equal(h.status.state, 'waiting');
  h.connections[1].send(live()); assert.equal(h.status.state, 'live');
});

test('initial construction failure retries without parallel connections', () => {
  let retries = 0;
  let pending;
  const client = createTelemetryClient({ onStatus: () => {},
    makeEventSource: () => { retries++; throw new Error('unavailable'); },
    schedule: (fn, ms) => { assert.equal(ms, 2000); pending = fn; return 1; }, cancel: () => {},
  });
  client.start(); assert.equal(retries, 1);
  pending(); assert.equal(retries, 2);
  client.stop();
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { createTelemetryClient } from '../web/telemetry-client.js';
import { isValidSnapshot, normalizeOptionalIndications } from '../web/telemetry/validation.js';
import { readFileSync } from 'node:fs';

const live = (sequence = 1, age = 0) => ({ schema_version: 1, sequence,
  source: 'demo', state: 'live', sample_age_ms: age,
  attitude: { pitch_deg: 5, roll_deg: 15 } });
const profile = JSON.parse(readFileSync(new URL('./fixtures/airspeed-profile.json', import.meta.url)));

function harness() {
  let time = 0;
  let nextId = 0;
  const timers = new Map();
  const connections = [];
  const statuses = [];
  const attitudes = [];
  const samples = [];
  const events = [];
  const client = createTelemetryClient({ onStatus: value => { statuses.push(value); events.push({ type: 'status', value }); },
    onAttitude: value => { attitudes.push(value); events.push({ type: 'attitude', value, status: statuses.at(-1) }); },
    onSample: value => { samples.push(value); events.push({ type: 'sample', value, status: statuses.at(-1) }); },
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
  return { client, connections, statuses, timers, attitudes, samples, events,
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
  for (const fixture of fixtures) assert.ok(isValidSnapshot(fixture));
  for (const invalid of [null, {}, { ...live(), schema_version: 2 },
    { ...live(), state: 'unknown' }, { ...live(), source: 'other' },
    { ...live(), sequence: 1.5 }, { ...live(), sequence: Number.MAX_SAFE_INTEGER + 1 },
    { ...live(), sample_age_ms: -1 }, { ...live(), sample_age_ms: null },
    { ...live(), attitude: { pitch_deg: 91, roll_deg: 0 } },
    { ...live(), attitude: { pitch_deg: 0, roll_deg: 180 } },
    { ...live(), attitude: { pitch_deg: NaN, roll_deg: 0 } },
    { ...live(), state: 'paused' }]) assert.ok(!isValidSnapshot(invalid));
});

test('optional values fail independently and never coerce missing values to zero', () => {
  assert.deepEqual(normalizeOptionalIndications(live()), { slip_skid: null, turn_rate_dps: null, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null });
  for (const invalid of [undefined, null, false, '0', {}, [], NaN, Infinity, -Infinity]) {
    assert.deepEqual(normalizeOptionalIndications({ slip_skid: invalid, turn_rate_dps: 3 }),
      { slip_skid: null, turn_rate_dps: 3, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null });
    assert.deepEqual(normalizeOptionalIndications({ slip_skid: -1, turn_rate_dps: invalid }),
      { slip_skid: -1, turn_rate_dps: null, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null });
  }
  for (const invalid of [-1.001, 1.001]) assert.equal(normalizeOptionalIndications({ slip_skid: invalid }).slip_skid, null);
  for (const row of JSON.parse(readFileSync(new URL('./fixtures/pfd-samples.json', import.meta.url)))) {
    assert.deepEqual(normalizeOptionalIndications(row), { slip_skid: row.slip_skid, turn_rate_dps: row.turn_rate_dps, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null });
  }
  assert.equal(normalizeOptionalIndications({ turn_rate_dps: 12 }).turn_rate_dps, 12);
});

test('new sample callback supports old snapshots and clears previously valid optional data', () => {
  const h = harness(); h.client.start();
  const connection = h.connections[0]; connection.onopen();
  connection.send({ ...live(1, 400), slip_skid: 1, turn_rate_dps: 3 });
  assert.deepEqual(h.samples.at(-1), { attitude: live().attitude, slip_skid: 1, turn_rate_dps: 3, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null, expiresAt: 600 });
  connection.send({ ...live(2), slip_skid: 'bad', turn_rate_dps: -3 });
  assert.equal(h.status.state, 'live');
  assert.equal(h.samples.at(-1).slip_skid, null);
  assert.equal(h.samples.at(-1).turn_rate_dps, -3);
  connection.send(live(3));
  assert.deepEqual(h.samples.at(-1), { attitude: live().attitude, slip_skid: null, turn_rate_dps: null, ias_kt: null, gs_kt: null, airspeed_profile: null, ias_acceleration_ktps: null, airspeed_profile: null, ias_acceleration_ktps: null, expiresAt: 1000 });
  connection.send({ ...live(4), schema_version: 2, turn_rate_dps: 3 });
  assert.equal(h.status.state, 'invalid');
  assert.equal(h.samples.length, 3);
});

test('accepted samples report current status before attitude and sample callbacks with separate copies', () => {
  const h = harness(); h.client.start();
  const connection = h.connections[0]; connection.onopen();
  connection.send(live(1));
  h.events.length = 0;
  h.advance(125);
  connection.send({ ...live(2, 400), source: 'simconnect', ias_kt: 1000,
    airspeed_profile: profile, ias_acceleration_ktps: -5 });
  const status = { transport: 'connected', source: 'simconnect', state: 'live' };
  assert.deepEqual(h.events.map(event => event.type), ['status', 'attitude', 'sample']);
  assert.deepEqual(h.events[0].value, status);
  assert.deepEqual(h.events[1].status, status);
  assert.deepEqual(h.events[2].status, status);
  assert.deepEqual(h.events[1].value, { attitude: { pitch_deg: 5, roll_deg: 15 }, expiresAt: 725 });
  assert.deepEqual(h.events[2].value, { attitude: { pitch_deg: 5, roll_deg: 15 },
    slip_skid: null, turn_rate_dps: null, ias_kt: 1000, gs_kt: null,
    airspeed_profile: profile, ias_acceleration_ktps: -5, expiresAt: 725 });
  assert.notStrictEqual(h.events[1].value.attitude, h.events[2].value.attitude);
  h.events[1].value.attitude.pitch_deg = 90;
  assert.deepEqual(h.events[2].value.attitude, { pitch_deg: 5, roll_deg: 15 });
  h.client.stop();
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

test('IAS and GS validate independently, preserve overflow values and clear omitted readings', () => {
  for (const invalid of [undefined, null, false, '0', {}, [], NaN, Infinity, -Infinity, -1]) {
    assert.equal(normalizeOptionalIndications({ ias_kt: invalid, gs_kt: 137 }).ias_kt, null);
    assert.equal(normalizeOptionalIndications({ ias_kt: invalid, gs_kt: 137 }).gs_kt, 137);
    assert.equal(normalizeOptionalIndications({ ias_kt: 123, gs_kt: invalid }).ias_kt, 123);
    assert.equal(normalizeOptionalIndications({ ias_kt: 123, gs_kt: invalid }).gs_kt, null);
  }
  for (const speed of [0, 99.5, 999, 1000]) {
    assert.equal(normalizeOptionalIndications({ ias_kt: speed, gs_kt: speed }).ias_kt, speed);
    assert.equal(normalizeOptionalIndications({ ias_kt: speed, gs_kt: speed }).gs_kt, speed);
  }
  const h = harness(); h.client.start();
  const connection = h.connections[0]; connection.onopen();
  connection.send({ ...live(1), ias_kt: 123, gs_kt: 137 });
  assert.equal(h.samples.at(-1).ias_kt, 123);
  h.advance(500);
  connection.send({ ...live(2), ias_kt: 123, gs_kt: 'bad' });
  assert.equal(h.samples.at(-1).expiresAt, 1500);
  assert.equal(h.samples.at(-1).ias_kt, 123);
  assert.equal(h.samples.at(-1).gs_kt, null);
  assert.equal(h.status.state, 'live');
  connection.send(live(3));
  assert.equal(h.samples.at(-1).ias_kt, null);
  assert.equal(h.samples.at(-1).gs_kt, null);
  h.advance(1000);
  assert.equal(h.status.state, 'stale');
});

test('profile and source acceleration retain their own availability through irregular receipt, expiry and recovery', () => {
  const h = harness(); h.client.start();
  let connection = h.connections[0]; connection.onopen();
  const extended = (sequence, age = 0) => ({ ...live(sequence, age), ias_kt: 110,
    gs_kt: 130, airspeed_profile: profile, ias_acceleration_ktps: 5 });
  for (const [sequence, delay] of [[1, 0], [2, 137], [3, 613]]) {
    h.advance(delay); connection.send(extended(sequence, 100));
    assert.equal(h.samples.at(-1).ias_acceleration_ktps, 5);
    assert.deepEqual(h.samples.at(-1).airspeed_profile, profile);
  }
  assert.equal(h.samples.at(-1).expiresAt, 1650);
  connection.send({ ...extended(4), ias_acceleration_ktps: '5' });
  assert.equal(h.samples.at(-1).ias_acceleration_ktps, null);
  assert.deepEqual(h.samples.at(-1).airspeed_profile, profile);
  connection.send({ ...extended(5), airspeed_profile: { ...profile, vne_kt: 0 } });
  assert.equal(h.samples.at(-1).airspeed_profile, null);
  assert.equal(h.samples.at(-1).ias_kt, 110);
  assert.equal(h.samples.at(-1).ias_acceleration_ktps, 5);
  connection.send({ ...extended(6), ias_kt: null, ias_acceleration_ktps: -5 });
  assert.equal(h.samples.at(-1).ias_kt, null);
  assert.deepEqual(h.samples.at(-1).airspeed_profile, profile);
  connection.send(live(7));
  assert.equal(h.samples.at(-1).airspeed_profile, null);
  assert.equal(h.samples.at(-1).ias_acceleration_ktps, null);
  connection.send(extended(8)); h.advance(1000);
  assert.equal(h.status.state, 'stale');
  const accepted = h.samples.length;
  connection.send(extended(9, 1000)); assert.equal(h.samples.length, accepted);
  connection.onerror(); h.advance(2000);
  assert.equal(h.connections.length, 2);
  connection = h.connections[1]; connection.onopen();
  assert.equal(h.status.state, 'waiting');
  connection.send(extended(1)); assert.equal(h.status.state, 'live');
  h.client.stop(); h.client.start(); h.client.start();
  assert.equal(h.connections.length, 3);
  assert.equal(h.status.state, 'waiting');
  h.connections[2].onopen(); h.connections[2].send(live(1));
  assert.equal(h.samples.at(-1).airspeed_profile, null);
  assert.equal(h.samples.at(-1).ias_acceleration_ktps, null);
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

test('attitude callback accepts only fresh new live samples with a monotonic deadline', () => {
  const h = harness(); h.client.start();
  const connection = h.connections[0]; connection.onopen();
  h.advance(100);
  connection.send(live(1, 400));
  assert.deepEqual(h.attitudes, [{ attitude: live().attitude, expiresAt: 700 }]);
  connection.send(live(1));
  connection.send(live(2, 1000));
  connection.receive({ data: 'broken JSON' });
  for (const [index, state] of ['waiting', 'paused', 'stale', 'disconnected', 'invalid'].entries()) {
    connection.send({ ...live(index + 3), state, attitude: null });
  }
  assert.equal(h.attitudes.length, 1);
  connection.send(live(8));
  assert.equal(h.attitudes.length, 2);
  assert.equal(h.attitudes.at(-1).expiresAt, 1100);
  h.client.stop(); connection.send(live(9));
  assert.equal(h.attitudes.length, 2);
  h.client.start(); h.connections[1].onopen();
  assert.equal(h.attitudes.length, 2);
  h.connections[1].send(live(1));
  assert.equal(h.attitudes.length, 3);
});

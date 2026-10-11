import { isValidSnapshot, normalizeOptionalIndications } from './validation.js';

// One read-only subscription, with independent source and transport status.
const STALE_AFTER = 1000;
const RETRY_AFTER = 2000;

export function createTelemetryClient({
  onStatus, onAttitude = () => {}, onSample = () => {}, makeEventSource = url => new EventSource(url),
  now = () => performance.now(), schedule = (fn, ms) => setTimeout(fn, ms),
  cancel = id => clearTimeout(id),
}) {
  let active = false;
  let connection = null;
  let retryTimer = null;
  let ageTimer = null;
  let sequence = -1;
  let status = { transport: 'connecting', source: null, state: 'waiting' };
  function report(update) {
    status = { ...status, ...update };
    onStatus({ ...status });
  }
  function clearAge() { cancel(ageTimer); ageTimer = null; }
  function readMessage(event) {
    try { return JSON.parse(event.data); } catch { return null; }
  }
  function expireSample(receivedAt, remaining) {
    const left = remaining - (now() - receivedAt);
    if (left > 0) {
      ageTimer = schedule(() => expireSample(receivedAt, remaining), left);
      return;
    }
    ageTimer = null;
    report({ state: 'stale' });
  }
  function publishSample(snapshot, expiresAt) {
    onAttitude({ attitude: { ...snapshot.attitude }, expiresAt });
    onSample({ attitude: { ...snapshot.attitude }, ...normalizeOptionalIndications(snapshot),
      expiresAt });
  }
  function acceptSample(snapshot) {
    if (!isValidSnapshot(snapshot)) {
      clearAge();
      report({ state: 'invalid' });
      return;
    }
    if (snapshot.sequence <= sequence) return;
    sequence = snapshot.sequence;
    clearAge();
    const receivedAt = now();
    const remaining = STALE_AFTER - (snapshot.sample_age_ms ?? 0);
    report({ source: snapshot.source,
      state: snapshot.state === 'live' && remaining <= 0 ? 'stale' : snapshot.state });
    if (status.state === 'live') {
      publishSample(snapshot, receivedAt + remaining);
      ageTimer = schedule(() => expireSample(receivedAt, remaining), remaining);
    }
  }
  function retry() {
    clearAge();
    if (connection) connection.close();
    connection = null;
    report({ transport: 'reconnecting', state: 'waiting' });
    if (active && retryTimer === null) {
      retryTimer = schedule(() => { retryTimer = null; connect(); }, RETRY_AFTER);
    }
  }
  function connect() {
    if (!active || connection) return;
    sequence = -1;
    try { connection = makeEventSource('/api/v1/events'); }
    catch { retry(); return; }
    const current = connection;
    current.onopen = () => {
      if (connection === current) report({ transport: 'connected', state: 'waiting' });
    };
    current.onerror = () => { if (connection === current) retry(); };
    current.addEventListener('telemetry', event => {
      if (connection !== current) return;
      acceptSample(readMessage(event));
    });
  }
  return {
    start() {
      if (active) return;
      active = true;
      report({ transport: 'connecting', state: 'waiting' });
      connect();
    },
    stop() {
      active = false;
      if (connection) connection.close();
      connection = null;
      cancel(retryTimer); retryTimer = null;
      clearAge();
      report({ transport: 'suspended', state: 'waiting' });
    },
  };
}

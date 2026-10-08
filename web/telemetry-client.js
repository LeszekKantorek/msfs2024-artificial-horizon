// One read-only subscription, with independent source and transport status.
const STATES = new Set(['waiting', 'live', 'paused', 'stale', 'disconnected', 'invalid']);
const STALE_AFTER = 1000;
const RETRY_AFTER = 2000;

export function validateSnapshot(value) {
  if (!value || value.schema_version !== 1 ||
      !Number.isSafeInteger(value.sequence) || value.sequence < 0 ||
      !['demo', 'simconnect'].includes(value.source) || !STATES.has(value.state) ||
      !(value.sample_age_ms === null ||
        (Number.isInteger(value.sample_age_ms) && value.sample_age_ms >= 0))) return false;
  if (value.state !== 'live') return value.attitude === null;
  const a = value.attitude;
  return value.sample_age_ms !== null && a !== null && typeof a === 'object' &&
    Number.isFinite(a.pitch_deg) && a.pitch_deg >= -90 && a.pitch_deg <= 90 &&
    Number.isFinite(a.roll_deg) && a.roll_deg >= -180 && a.roll_deg < 180;
}

export function createTelemetryClient({
  onStatus, makeEventSource = url => new EventSource(url),
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
      let snapshot;
      try { snapshot = JSON.parse(event.data); } catch { snapshot = null; }
      if (!validateSnapshot(snapshot)) {
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
        const expire = () => {
          const left = remaining - (now() - receivedAt);
          if (left > 0) { ageTimer = schedule(expire, left); return; }
          ageTimer = null;
          report({ state: 'stale' });
        };
        ageTimer = schedule(expire, remaining);
      }
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

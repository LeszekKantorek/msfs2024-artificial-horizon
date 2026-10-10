import test from 'node:test';
import assert from 'node:assert/strict';
import { composePanel } from '../web/panel.js';
import { createFrameScheduler } from '../web/frame-scheduler.js';
import { createStatusView } from '../web/status.js';
import { slipPosition } from '../web/slip-skid.js';
import { turnGeometry } from '../web/turn-rate.js';
import { pfdLayout } from '../web/layout.js';

test('all render methods receive the same complete read-only frame', () => {
  const received = [], resized = [], invalidated = [];
  const instruments = Array.from({ length: 3 }, () => ({
    resize: layout => resized.push(layout), render: frame => received.push(frame),
  }));
  const frame = Object.freeze({
    attitude: Object.freeze({ pitch_deg: 10, roll_deg: 25 }), slip_skid: -1,
    turn_rate_dps: -3, expiresAt: 500,
    status: Object.freeze({ source: 'demo', transport: 'connected', state: 'live' }),
    futureField: 123,
  });
  const attributes = {};
  const panel = composePanel({ setAttribute: (name, value) => { attributes[name] = value; } }, instruments, {
    render: frame => received.push(frame), invalidate: reason => invalidated.push(reason),
  });
  const layout = pfdLayout(320, 440);
  panel.resize(layout); panel.render(frame); panel.invalidate('stale');
  assert.equal(attributes.viewBox, '0 0 320 440');
  assert.equal(received.length, 4);
  assert.ok(received.every(value => value === frame));
  assert.ok(resized.every(value => value === layout));
  assert.deepEqual(invalidated, ['stale']);
  assert.equal(frame.futureField, 123);
});

function statusHarness() {
  const elements = new Map();
  for (const selector of ['.source', '#transport', '#data-status', '.instrument', '#unavailable']) {
    let text = '', writes = 0;
    elements.set(selector, { dataset: { available: 'false' },
      get textContent() { return text; },
      set textContent(value) { text = value; writes++; },
      get writes() { return writes; },
    });
  }
  const view = createStatusView({ querySelector: selector => elements.get(selector) });
  return { view, get: selector => elements.get(selector) };
}
const liveStatus = { source: 'demo', transport: 'connected', state: 'live' };

test('status without a sample stays obscured and unchanged labels are not announced again', () => {
  const h = statusHarness();
  h.view.update(liveStatus);
  assert.equal(h.get('.instrument').dataset.available, 'false');
  const writes = h.get('#data-status').writes;
  const frame = { status: liveStatus };
  h.view.render(frame); h.view.render(frame);
  assert.equal(h.get('.instrument').dataset.available, 'true');
  assert.equal(h.get('#data-status').writes, writes);
  h.view.update({ ...liveStatus, transport: 'reconnecting', state: 'waiting' });
  h.view.invalidate('reconnecting');
  assert.equal(h.get('.source').textContent, 'DEMO');
  assert.equal(h.get('#data-status').textContent, 'Waiting for fresh telemetry');
  assert.equal(h.get('#transport').textContent, 'Connection lost. Retrying…');
  assert.equal(h.get('.instrument').dataset.available, 'false');
});

test('expiry during resize obscures the panel without painting or restoring an old frame', () => {
  const h = statusHarness();
  let now = 0, callback;
  const drawn = [], resized = [];
  const panel = composePanel({ setAttribute() {} }, [{
    resize: layout => resized.push(layout), render: frame => drawn.push(frame),
  }], h.view);
  const scheduler = createFrameScheduler({ panel, now: () => now,
    requestFrame: fn => { callback = fn; return 1; }, cancelFrame() {},
  });
  scheduler.resize(pfdLayout(320, 440));
  scheduler.accept({ attitude: { pitch_deg: 10, roll_deg: 25 }, expiresAt: 500, status: liveStatus });
  callback();
  assert.equal(drawn.length, 1);
  now = 500;
  scheduler.resize(pfdLayout(568, 204)); callback();
  assert.equal(drawn.length, 1);
  assert.equal(h.get('.instrument').dataset.available, 'false');
  assert.equal(h.get('#data-status').dataset.state, 'stale');
  assert.equal(h.get('#unavailable').textContent, 'Telemetry stale');
  scheduler.resize(pfdLayout(320, 440));
  assert.equal(drawn.length, 1);
  assert.equal(resized.length, 3);
});

test('local slip and turn geometry matches independent direction and range expectations', () => {
  assert.deepEqual([-1, 0, 1].map(value => slipPosition(value, 28)), [-28, 0, 28]);
  for (const [value, end, overflow] of [[-9, -80, true], [-6, -80, false],
    [-3, -40, false], [0, 0, false], [3, 40, false], [6, 80, false], [9, 80, true]]) {
    const geometry = turnGeometry(value, 80);
    assert.equal(geometry.end, end);
    assert.equal(geometry.vector, `M 0 0 H ${end}`);
    assert.equal(geometry.overflow !== '', overflow);
    if (overflow) assert.equal(geometry.overflow, `M ${end} 0 l ${value < 0 ? 6 : -6} -5 v 10 Z`);
  }
});

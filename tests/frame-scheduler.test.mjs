import test from 'node:test';
import assert from 'node:assert/strict';
import { createFrameScheduler } from '../web/frame-scheduler.js';
import { pfdLayout } from '../web/layout.js';

function harness() {
  let time = 0;
  let nextId = 0;
  const frames = new Map();
  const drawn = [];
  const raw = [], accepted = [];
  const obscured = [];
  const renderer = createFrameScheduler({
    panel: { render: (value, time, sample) => { drawn.push(value); raw.push(sample); },
      accept: (sample, time) => accepted.push([sample, time]), invalidate: reason => obscured.push(reason), resize() {} },
    now: () => time,
    requestFrame: fn => { const id = ++nextId; frames.set(id, fn); return id; },
    cancelFrame: id => frames.delete(id),
  });
  return { renderer, frames, drawn, raw, accepted, obscured,
    advance: ms => { time += ms; },
    flush() { const pending = [...frames.values()]; frames.clear(); for (const fn of pending) fn(); },
  };
}
const sample = (pitch = 0, roll = 0, expiresAt = 1000) => ({
  attitude: { pitch_deg: pitch, roll_deg: roll }, expiresAt,
});

test('redraw reuses the existing frame and never extends its deadline', () => {
  const h = harness();
  const frame = sample(0, 0, 500);
  h.renderer.accept(frame); h.flush();
  h.advance(499); h.renderer.redraw(); h.flush();
  assert.deepEqual(h.drawn, [frame, frame]);
  h.advance(1); h.renderer.redraw(); h.flush();
  assert.deepEqual(h.obscured, ['stale']);
  assert.equal(h.drawn.length, 2);
});

test('headings draw with the newest attitude and expire before resize can renew them', () => {
  const h = harness();
  h.renderer.accept({ ...sample(0, 0, 500), heading_deg: 359, selected_heading_deg: 1 });
  const latest = { ...sample(5, 15, 500), heading_deg: 0, selected_heading_deg: 1 };
  h.renderer.accept(latest); h.flush(); assert.equal(h.drawn[0].attitude.pitch_deg, 0); assert.equal(h.drawn[0].heading_deg, 359);
  h.advance(499); h.renderer.resize(pfdLayout(568, 240)); h.flush();
  assert.deepEqual(h.drawn.at(-1), latest);
  h.advance(1); h.renderer.resize(pfdLayout(320, 480)); h.flush();
  assert.equal(h.drawn.length, 2); assert.deepEqual(h.obscured, ['stale']);
});

test('speed indications stay in the same latest frame and retain their deadline through resize and expiry', () => {
  const h = harness();
  const first = { ...sample(0, 0, 500), ias_kt: 110, tas_kt: 120, gs_kt: 130, ias_acceleration_ktps: 5,
    airspeed_profile: { vne_kt: 140 } };
  const latest = { ...first, ias_kt: 150, tas_kt: 160, gs_kt: 170, ias_acceleration_ktps: 0 };
  h.renderer.accept(first); h.renderer.accept(latest); h.flush();
  assert.equal(h.drawn.length, 1);
  assert.equal(h.drawn[0].ias_kt, 110); assert.equal(h.drawn[0].tas_kt, 160);
  h.advance(499); h.renderer.resize(pfdLayout(568, 240)); h.flush();
  assert.deepEqual(h.drawn.at(-1), latest);
  h.advance(1); h.renderer.resize(pfdLayout(320, 480)); h.flush();
  assert.equal(h.drawn.length, 2);
  assert.deepEqual(h.obscured, ['stale']);
  h.renderer.accept({ ...first, expiresAt: 500 }); h.flush();
  assert.equal(h.drawn.length, 2);
});

test('altitude and pressure coalesce together and expire before a resize draw', () => {
  const h = harness();
  h.renderer.accept({ ...sample(0, 0, 500), altitude_ft: -20, baro_text: '29.92 inHg' });
  const latest = { ...sample(0, 0, 500), altitude_ft: 20, baro_text: '30.12 inHg' };
  h.renderer.accept(latest); h.flush();
  assert.notDeepEqual(h.drawn[0], latest);
  h.advance(499); h.renderer.resize(pfdLayout(568, 240)); h.flush();
  assert.deepEqual(h.drawn.at(-1), latest);
  h.advance(1); h.renderer.resize(pfdLayout(320, 480)); h.flush();
  assert.equal(h.drawn.length, 2);
  assert.deepEqual(h.obscured, ['stale']);
});

test('VSI coalesces with altitude and target, retaining the shared deadline through resize', () => {
  const h = harness();
  h.renderer.accept({ ...sample(0, 0, 500), altitude_ft: 8500, selected_altitude_ft: 10000, vertical_speed_fpm: 0 });
  const latest = { ...sample(0, 0, 500), altitude_ft: 8525, selected_altitude_ft: 10000, vertical_speed_fpm: 1500 };
  h.renderer.accept(latest); h.flush();
  assert.notDeepEqual(h.drawn[0], latest);
  h.advance(499); h.renderer.resize(pfdLayout(568, 240)); h.flush();
  assert.deepEqual(h.drawn.at(-1), latest);
  h.advance(1); h.renderer.resize(pfdLayout(320, 480)); h.flush();
  assert.equal(h.drawn.length, 2);
  assert.deepEqual(h.obscured, ['stale']);
});

test('a burst latches its first sample and animates to the newest target', () => {
  const h = harness();
  h.renderer.accept(sample(15)); h.renderer.accept(sample(-15)); h.renderer.accept(sample(10, 25));
  assert.equal(h.frames.size, 1); assert.equal(h.drawn.length, 0);
  h.flush();
  assert.deepEqual(h.drawn, [sample(15)]);
  assert.equal(h.frames.size, 1);
  h.advance(16); h.flush(); assert.deepEqual(h.drawn.at(-1), sample(10, 25));
  assert.equal(h.frames.size, 0);
  h.renderer.accept(sample()); h.flush(); assert.equal(h.drawn.length, 3);
});

test('a sample expiring before its frame is obscured even if timeout delivery is delayed', () => {
  const h = harness(); h.renderer.accept(sample(15, 30, 500));
  h.advance(500); h.flush();
  assert.equal(h.drawn.length, 0); assert.deepEqual(h.obscured, ['stale']);
  h.renderer.accept(sample(0, 0, 1500)); h.flush(); assert.equal(h.drawn.length, 1);
});

test('unavailable and suspended states cancel pending rendering and discard the old sample', () => {
  for (const reason of ['connecting', 'reconnecting', 'suspended', 'waiting', 'paused', 'stale', 'disconnected', 'invalid']) {
    const h = harness(); h.renderer.accept(sample(10, 25));
    const delayed = [...h.frames.values()][0];
    h.renderer.invalidate(reason); h.flush(); delayed();
    assert.equal(h.frames.size, 0); assert.equal(h.drawn.length, 0);
    assert.deepEqual(h.obscured, [reason]);
    h.renderer.accept(sample(-10, -25)); h.flush(); assert.equal(h.drawn.length, 1);
  }
});

test('resize preserves the last values and deadline, coalescing with new samples', () => {
  const h = harness();
  h.renderer.accept(sample(10, 25, 500)); h.flush();
  const portrait = pfdLayout(320, 440);
  h.renderer.resize(portrait); h.renderer.resize(pfdLayout(568, 204));
  assert.equal(h.frames.size, 1);
  h.flush();
  assert.equal(h.drawn.length, 2);
  const last = h.drawn.at(-1);
  assert.deepEqual(last.attitude, { pitch_deg: 10, roll_deg: 25 });
  assert.equal(last.expiresAt, 500);
  assert.equal(h.frames.size, 0);
  h.advance(500);
  h.renderer.resize(portrait); h.flush();
  assert.equal(h.drawn.length, 2);
  assert.deepEqual(h.obscured, ['stale']);
  h.renderer.resize(portrait); h.flush();
  assert.equal(h.frames.size, 0);
});

test('an already cancelled frame cannot draw a newer subscription sample', () => {
  const h = harness(); h.renderer.accept(sample(10, 25));
  const oldCallback = [...h.frames.values()][0];
  h.renderer.invalidate('reconnecting');
  h.renderer.accept(sample(-10, -25));
  oldCallback();
  assert.equal(h.drawn.length, 0);
  assert.equal(h.frames.size, 1);
  h.flush(); assert.equal(h.drawn.length, 1);
});

test('fresh identical values replace the deadline even with an existing pending draw', () => {
  const h = harness();
  h.renderer.accept(sample(10, 25, 500));
  h.advance(400);
  h.renderer.accept(sample(10, 25, 1400));
  h.advance(200); h.flush();
  assert.equal(h.drawn.length, 1);
  assert.equal(h.drawn[0].expiresAt, 1400);
});

test('resize updates geometry immediately while unavailable and never restores discarded data', () => {
  const resized = [], drawn = [], obscured = [];
  const renderer = createFrameScheduler({
    panel: { resize: layout => resized.push(layout), render: frame => { drawn.push(frame); },
      invalidate: reason => obscured.push(reason) },
    requestFrame() { assert.fail('Unavailable resize must not schedule a draw'); },
  });
  const layout = pfdLayout(320, 440);
  renderer.invalidate('suspended'); renderer.resize(layout);
  assert.deepEqual(resized, [layout]); assert.deepEqual(drawn, []);
  assert.deepEqual(obscured, ['suspended']);
});

test('resize and fresh values commit together using the newest layout and sample', () => {
  const events = [], frames = [];
  const scheduler = createFrameScheduler({
    panel: { resize: layout => events.push(['resize', layout]),
      render: frame => { events.push(['render', frame]); }, invalidate() {} },
    now: () => 0, requestFrame: fn => { frames.push(fn); return frames.length; },
  });
  scheduler.accept(sample(10)); frames.shift()(); events.length = 0;
  const layout = pfdLayout(568, 240);
  scheduler.resize(pfdLayout(320, 480)); scheduler.resize(layout);
  scheduler.accept(sample(15));
  assert.deepEqual(events, []);
  assert.equal(frames.length, 1);
  frames.shift()();
  assert.deepEqual(events, [['resize', layout], ['render', sample(10)]]);
});

test('invalidation applies pending geometry under the cover and discards pending values', () => {
  const events = [];
  let callback;
  const scheduler = createFrameScheduler({
    panel: { resize: layout => events.push(['resize', layout]),
      render: () => { events.push(['render']); }, invalidate: reason => events.push(['cover', reason]) },
    requestFrame: fn => { callback = fn; return 1; }, cancelFrame() {}, now: () => 0,
  });
  scheduler.accept(sample()); callback(); events.length = 0;
  const layout = pfdLayout(568, 240);
  scheduler.resize(layout);
  scheduler.invalidate('suspended'); callback();
  assert.deepEqual(events, [['cover', 'suspended'], ['resize', layout]]);
});

test('motion interpolates at receipt cadence and retargets from the last paint', () => {
  const h = harness();
  h.renderer.accept(sample(0, 0, 2000)); h.flush();
  h.advance(330); h.renderer.accept(sample(30, 0, 2000)); h.flush();
  assert.equal(h.drawn.at(-1).attitude.pitch_deg, 0);
  h.advance(165); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 15);
  h.advance(1); h.renderer.accept(sample(45, 0, 2000));
  h.advance(15); h.flush(); assert.ok(Math.abs(h.drawn.at(-1).attitude.pitch_deg - 17.891566) < 0.00001);
});

test('fresh recovery baseline is drawn before a coalesced target', () => {
  const h = harness();
  h.advance(100); h.renderer.accept(sample(0, 0, 1000));
  h.advance(50); h.renderer.accept(sample(30, 0, 1000));
  h.advance(10); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 0);
  h.advance(25); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 15);
  h.advance(25); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 30);
  assert.equal(h.frames.size, 0);
});

test('roll crosses its wrap by the shortest path', () => {
  const h = harness(); h.renderer.accept(sample(0, 179, 1000)); h.flush();
  h.advance(100); h.renderer.accept(sample(0, -179, 1000)); h.flush();
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).attitude.roll_deg, -180);
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).attitude.roll_deg, -179);
});


test('channel recovery coalesces to the newest eligible value after loss', () => {
  const h = harness();
  const speed = value => ({ ...sample(0, 0, 2000), ias_kt: value });
  h.renderer.accept(speed(100)); h.flush();
  h.advance(10); h.renderer.accept(speed(null));
  h.advance(10); h.renderer.accept(speed(120));
  h.advance(10); h.renderer.accept(speed(140)); h.flush();
  assert.equal(h.drawn.at(-1).ias_kt, 140);
  assert.equal(h.frames.size, 0);
});

test('repeated eligibility breaks before the first draw snap to the latest recovery', () => {
  const h = harness();
  const speed = value => ({ ...sample(0, 0, 2000), ias_kt: value });
  h.renderer.accept(speed(100)); h.advance(10); h.renderer.accept(speed(null));
  h.advance(10); h.renderer.accept(speed(120)); h.advance(10); h.renderer.accept(speed(1000));
  h.advance(10); h.renderer.accept(speed(130)); h.advance(10); h.renderer.accept(speed(140)); h.flush();
  assert.equal(h.drawn.at(-1).ias_kt, 140);
  assert.equal(h.frames.size, 0);
});

test('equal targets keep timing and raw expiry; an active target survives a coalesced detour', () => {
  const h = harness(); h.renderer.accept(sample(0, 0, 2000)); h.flush();
  h.advance(100); h.renderer.accept(sample(20, 0, 2000));
  h.advance(10); h.renderer.accept(sample(20, 0, 2100)); h.flush();
  assert.equal(h.drawn.at(-1).attitude.pitch_deg, 2);
  assert.equal(h.drawn.at(-1).expiresAt, 2100);
  h.advance(10); h.renderer.accept(sample(30, 0, 2100));
  h.advance(10); h.renderer.accept(sample(20, 0, 2200)); h.flush();
  assert.equal(h.drawn.at(-1).attitude.pitch_deg, 6);
  h.advance(70); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 20);
});

test('expired recovery baseline is skipped in favor of a fresh target', () => {
  const h = harness(); h.renderer.accept(sample(0, 0, 50));
  h.advance(40); h.renderer.accept(sample(30, 0, 1000));
  h.advance(20); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 30);
  assert.equal(h.frames.size, 0);
});

test('heading takes the shortest path across north and a negative 180-degree tie', () => {
  const h = harness(); const make = value => ({ ...sample(0, 0, 2000), heading_deg: value });
  h.renderer.accept(make(359)); h.flush(); h.advance(100); h.renderer.accept(make(1)); h.flush();
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).heading_deg, 0);
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).heading_deg, 1);
  h.advance(100); h.renderer.accept(make(181)); h.flush(); h.advance(100); h.flush();
  assert.equal(h.drawn.at(-1).heading_deg, 271);
});

test('turn overflow pins immediately and range recovery moves from the drawn edge', () => {
  const h = harness(); const make = value => ({ ...sample(0, 0, 2000), turn_rate_dps: value });
  h.renderer.accept(make(0)); h.flush(); h.advance(100); h.renderer.accept(make(9)); h.flush();
  assert.equal(h.drawn.at(-1).turn_rate_dps, 6);
  h.advance(100); h.renderer.accept(make(-9)); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, -6);
  h.advance(100); h.renderer.accept(make(-3)); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, -6);
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, -4.5);
});


test('an unpainted overflow in the recovery baseline snaps to the latest in-range turn', () => {
  const make = value => ({ ...sample(0, 0, 2000), turn_rate_dps: value });
  for (const values of [[9, 3], [0, 9, 3]]) {
    const h = harness();
    for (const value of values) { h.renderer.accept(make(value)); h.advance(10); }
    h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, 3);
    assert.equal(h.frames.size, 0);
  }
});

test('coalesced turn overflow after an in-range paint keeps its actual anchor', () => {
  const h = harness(); const make = value => ({ ...sample(0, 0, 2000), turn_rate_dps: value });
  h.renderer.accept(make(0)); h.flush(); h.advance(100); h.renderer.accept(make(9));
  h.advance(10); h.renderer.accept(make(3)); h.flush();
  assert.equal(h.drawn.at(-1).turn_rate_dps, 0);
  h.advance(16); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, 3);
});


test('turn re-entry at the capped value uses its own receipt timing', () => {
  const h = harness(); const make = value => ({ ...sample(0, 0, 1000), turn_rate_dps: value });
  h.renderer.accept(make(2)); h.flush(); h.advance(100); h.renderer.accept(make(9));
  h.advance(10); h.renderer.accept(make(6)); h.flush();
  assert.equal(h.drawn.at(-1).turn_rate_dps, 2);
  h.advance(8); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, 4);
  h.advance(8); h.flush(); assert.equal(h.drawn.at(-1).turn_rate_dps, 6);
});


test('raw sample and per-message history stay separate from moving BARO, VSI and altitude', () => {
  const h = harness();
  const first = { ...sample(0, 0, 1000), altitude_ft: 1000, selected_altitude_ft: 1200,
    vertical_speed_fpm: 1000, baro_text: '29.92 inHg' };
  const next = { ...first, altitude_ft: 1100, vertical_speed_fpm: 2000, baro_text: '30.12 inHg', expiresAt: 1500 };
  h.renderer.accept(first); h.flush(); h.advance(100); h.renderer.accept(next); h.flush();
  assert.deepEqual(h.accepted, [[first, 0], [next, 100]]); assert.equal(h.raw.at(-1), next);
  assert.equal(h.drawn.at(-1).altitude_ft, 1000); assert.equal(h.drawn.at(-1).vertical_speed_fpm, 1000);
  assert.equal(h.drawn.at(-1).baro_text, '30.12 inHg'); assert.equal(h.drawn.at(-1).expiresAt, 1500);
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).altitude_ft, 1050);
  assert.equal(h.drawn.at(-1).vertical_speed_fpm, 1500);
});

test('a new target after baseline draw uses its own receipt and duration', () => {
  const h = harness(); h.advance(100); h.renderer.accept(sample(0, 0, 1000));
  h.advance(50); h.renderer.accept(sample(30, 0, 1000)); h.advance(10); h.flush();
  h.advance(10); h.renderer.accept(sample(40, 0, 1000)); h.flush();
  assert.equal(h.drawn.at(-1).attitude.pitch_deg, 0);
  h.advance(10); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 20);
  h.advance(10); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 40);
});

test('settled coalesced targets use the last change and consecutive retargets progress', () => {
  const h = harness(); h.renderer.accept(sample(0, 0, 2000)); h.flush();
  h.advance(100); h.renderer.accept(sample(10, 0, 2000)); h.advance(10); h.renderer.accept(sample(20, 0, 2000)); h.flush();
  assert.equal(h.drawn.at(-1).attitude.pitch_deg, 0);
  h.advance(8); h.flush(); assert.equal(h.drawn.at(-1).attitude.pitch_deg, 10);
  h.advance(7); h.renderer.accept(sample(30, 0, 2000)); h.advance(1); h.flush();
  const first = h.drawn.at(-1).attitude.pitch_deg; assert.ok(first > 10);
  h.advance(15); h.renderer.accept(sample(40, 0, 2000)); h.advance(1); h.flush();
  assert.ok(h.drawn.at(-1).attitude.pitch_deg > first);
});

test('opposite finite extreme trend rates blend without overflow', () => {
  const h = harness(); const make = rate => ({ ...sample(0, 0, 2000), altitude_ft: 10000, vertical_speed_fpm: rate });
  h.renderer.accept(make(1e308)); h.flush(); h.advance(100); h.renderer.accept(make(-1e308)); h.flush();
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).vertical_speed_fpm, 0);
});

test('reverse heading and roll wraps cross their normalized boundary', () => {
  const h = harness(); const make = (heading, roll) => ({ ...sample(0, roll, 2000), heading_deg: heading });
  h.renderer.accept(make(1, -179)); h.flush(); h.advance(100); h.renderer.accept(make(359, 179)); h.flush();
  h.advance(50); h.flush(); assert.equal(h.drawn.at(-1).heading_deg, 0); assert.equal(h.drawn.at(-1).attitude.roll_deg, -180);
});

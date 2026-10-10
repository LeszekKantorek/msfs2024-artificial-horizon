import test from 'node:test';
import assert from 'node:assert/strict';
import { createFrameScheduler } from '../web/frame-scheduler.js';
import { pfdLayout } from '../web/layout.js';

function harness() {
  let time = 0;
  let nextId = 0;
  const frames = new Map();
  const drawn = [];
  const obscured = [];
  const renderer = createFrameScheduler({
    panel: { render: value => drawn.push(value), invalidate: reason => obscured.push(reason), resize() {} },
    now: () => time,
    requestFrame: fn => { const id = ++nextId; frames.set(id, fn); return id; },
    cancelFrame: id => frames.delete(id),
  });
  return { renderer, frames, drawn, obscured,
    advance: ms => { time += ms; },
    flush() { const pending = [...frames.values()]; frames.clear(); for (const fn of pending) fn(); },
  };
}
const sample = (pitch = 0, roll = 0, expiresAt = 1000) => ({
  attitude: { pitch_deg: pitch, roll_deg: roll }, expiresAt,
});

test('a burst draws only its newest sample in one frame, without continuous animation', () => {
  const h = harness();
  h.renderer.accept(sample(15)); h.renderer.accept(sample(-15)); h.renderer.accept(sample(10, 25));
  assert.equal(h.frames.size, 1); assert.equal(h.drawn.length, 0);
  h.flush();
  assert.deepEqual(h.drawn, [sample(10, 25)]);
  assert.equal(h.frames.size, 0);
  h.renderer.accept(sample()); h.flush(); assert.equal(h.drawn.length, 2);
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
    panel: { resize: layout => resized.push(layout), render: frame => drawn.push(frame),
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
      render: frame => events.push(['render', frame]), invalidate() {} },
    now: () => 0, requestFrame: fn => { frames.push(fn); return frames.length; },
  });
  scheduler.accept(sample(10)); frames.shift()(); events.length = 0;
  const layout = pfdLayout(568, 240);
  scheduler.resize(pfdLayout(320, 480)); scheduler.resize(layout);
  scheduler.accept(sample(15));
  assert.deepEqual(events, []);
  assert.equal(frames.length, 1);
  frames.shift()();
  assert.deepEqual(events, [['resize', layout], ['render', sample(15)]]);
});

test('invalidation applies pending geometry under the cover and discards pending values', () => {
  const events = [];
  let callback;
  const scheduler = createFrameScheduler({
    panel: { resize: layout => events.push(['resize', layout]),
      render: () => events.push(['render']), invalidate: reason => events.push(['cover', reason]) },
    requestFrame: fn => { callback = fn; return 1; }, cancelFrame() {}, now: () => 0,
  });
  scheduler.accept(sample()); callback(); events.length = 0;
  const layout = pfdLayout(568, 240);
  scheduler.resize(layout);
  scheduler.invalidate('suspended'); callback();
  assert.deepEqual(events, [['cover', 'suspended'], ['resize', layout]]);
});

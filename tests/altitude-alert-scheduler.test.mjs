import test from 'node:test';
import assert from 'node:assert/strict';
import { createFrameScheduler } from '../web/frame-scheduler.js';
import { initialAlert, advanceAlert, alertPresentation } from '../web/altitude/selected.js';
import { composePanel } from '../web/panel.js';
import { pfdLayout } from '../web/layout.js';

function harness() {
  let time = 0, id = 0, alert = initialAlert();
  const timers = new Map(), frames = new Map(), accepted = [], draws = [], covers = [], layouts = [];
  const panel = composePanel({ setAttribute() {}, querySelector: () => ({ setAttribute() {} }) }, [{
    resize: layout => layouts.push(layout),
    accept(frame, now) { accepted.push(frame); alert = advanceAlert(alert, frame.altitude_ft, frame.selected_altitude_ft, now); },
    render(frame, now) { const visual = alertPresentation(alert, now); draws.push({ frame, now, ...visual }); return visual.nextUpdateAt; },
    invalidate() { alert = initialAlert(); },
  }], { render() {}, invalidate: reason => covers.push(reason) });
  const scheduler = createFrameScheduler({ panel, now: () => time,
    setTimer: (fn, ms) => { const key = ++id; timers.set(key, { fn, at: time + ms }); return key; },
    clearTimer: key => timers.delete(key),
    requestFrame: fn => { const key = ++id; frames.set(key, fn); return key; }, cancelFrame: key => frames.delete(key),
  });
  return { scheduler, accepted, draws, covers, layouts, timers, frames,
    get alert() { return alert; },
    accept(altitude_ft, selected_altitude_ft = 10000, expiresAt = time + 1000) {
      scheduler.accept(Object.freeze({ altitude_ft, selected_altitude_ft, expiresAt }));
    },
    setTime(value) { time = value; },
    tick(value) {
      time = value;
      for (const [key, timer] of [...timers]) if (timer.at <= time) { timers.delete(key); timer.fn(); }
    },
    flush() { const callbacks = [...frames.values()]; frames.clear(); for (const fn of callbacks) fn(); },
  };
}

test('all samples advance alerts before one newest-sample draw; resize cannot restart flashing', () => {
  const h = harness();
  h.accept(8500); h.setTime(10); h.accept(9000); h.setTime(20); h.accept(10000);
  assert.equal(h.accepted.length, 3); assert.equal(h.frames.size, 1); assert.equal(h.draws.length, 0);
  h.flush(); assert.equal(h.draws[0].frame.altitude_ft, 10000); assert.equal(h.alert.flashStarted, 20);
  h.setTime(270); h.scheduler.resize(pfdLayout(320, 400)); h.flush();
  assert.equal(h.draws.at(-1).visible, false); assert.equal(h.draws.at(-1).frame.expiresAt, 1020);
  assert.equal(h.alert.flashStarted, 20); assert.equal(h.accepted.length, 3);
  assert.equal(h.timers.size, 1);
});

test('timers only enqueue the common frame and delayed callbacks use absolute flash time', () => {
  const h = harness(); h.accept(8500); h.accept(9000); h.flush();
  h.tick(250); assert.equal(h.draws.length, 1); assert.equal(h.frames.size, 1);
  h.flush(); assert.equal(h.draws.at(-1).visible, false);
  h.tick(900); h.flush(); assert.equal(h.draws.at(-1).visible, false);
  assert.equal(h.draws.at(-1).now, 900); assert.equal([...h.timers.values()][0].at, 1000);
});

test('expiry during flashing obscures data before drawing, including a delayed frame', () => {
  const h = harness(); h.accept(8500); h.accept(9000); h.flush();
  h.tick(250); h.setTime(1000); h.flush();
  assert.deepEqual(h.covers, ['stale']); assert.equal(h.draws.length, 1);
  assert.equal(h.timers.size, 0); assert.equal(h.frames.size, 0); assert.deepEqual(h.alert, initialAlert());
  h.accept(9999); h.flush(); assert.equal(h.draws.at(-1).flashing, false);
});

test('all invalidation reasons cancel work and old callbacks cannot disturb a resumed alert', () => {
  for (const reason of ['paused', 'stale', 'disconnected', 'reconnecting', 'suspended', 'invalid', 'waiting']) {
    const h = harness(); h.accept(8500); h.accept(9000); h.flush();
    const oldTimer = [...h.timers.values()][0].fn;
    h.scheduler.resize(pfdLayout(320, 400)); const oldFrame = [...h.frames.values()][0];
    h.scheduler.invalidate(reason);
    assert.equal(h.timers.size, 0); assert.equal(h.frames.size, 0); assert.deepEqual(h.alert, initialAlert());
    h.accept(8500); h.accept(9000); h.flush();
    const timer = [...h.timers.keys()][0];
    oldTimer(); oldFrame();
    assert.equal(h.frames.size, 0); assert.deepEqual([...h.timers.keys()], [timer]);
    assert.deepEqual(h.covers, [reason]);
  }
});

test('target change and local loss cancel old timing with silent recovery', () => {
  const h = harness(); h.accept(8500); h.accept(9000); h.flush();
  const oldTimer = [...h.timers.values()][0].fn;
  h.accept(9000, 9000); h.flush(); assert.equal(h.timers.size, 0);
  h.accept(9300, 9000); h.flush(); assert.equal(h.draws.at(-1).deviation, true);
  h.accept(null, 9000); h.flush(); assert.equal(h.draws.at(-1).flashing, false);
  assert.equal(h.timers.size, 0); oldTimer(); assert.equal(h.frames.size, 0);
  h.accept(9300, 9000); h.flush(); assert.equal(h.draws.at(-1).deviation, false);
  h.accept(9000, null); h.flush(); assert.deepEqual(h.alert, initialAlert());
});

test('fresh identical values renew only freshness and flash finishes exactly once', () => {
  const h = harness(); h.accept(8500); h.accept(9000); h.flush();
  for (let time = 500; time <= 5000; time += 500) {
    h.setTime(time); h.accept(9000); h.flush();
    assert.equal(h.alert.flashStarted, 0); assert.equal(h.timers.size, time < 5000 ? 1 : 0);
  }
  assert.equal(h.draws.at(-1).flashing, false); assert.equal(h.draws.at(-1).visible, true);
});

test('expired acceptance never updates alert history and early timers preserve the deadline', () => {
  const h = harness(); h.setTime(1000); h.accept(9000, 10000, 1000);
  assert.equal(h.accepted.length, 0); assert.deepEqual(h.covers, ['stale']);
  h.accept(8500); h.accept(9000); h.flush();
  const early = [...h.timers.values()][0].fn; early(); h.flush();
  assert.equal(h.draws.at(-1).visible, true); assert.equal([...h.timers.values()][0].at, 1250);
});

test('a fresh sample after delayed expiry resets history before establishing its silent baseline', () => {
  const h = harness(); h.accept(10000); h.accept(10201); h.flush();
  const oldTimer = [...h.timers.values()][0].fn;
  h.scheduler.resize(pfdLayout(320, 400)); const oldFrame = [...h.frames.values()][0];
  h.setTime(1500); h.accept(10201); h.flush();
  assert.deepEqual(h.covers, ['stale']);
  assert.equal(h.draws.at(-1).deviation, false); assert.equal(h.draws.at(-1).flashing, false);
  assert.equal(h.alert.captured, false); assert.equal(h.timers.size, 0);
  oldTimer(); oldFrame(); assert.equal(h.frames.size, 0);
  h.accept(10000); h.flush(); assert.equal(h.alert.captured, true);
  h.accept(10201); h.flush(); assert.equal(h.draws.at(-1).deviation, true);
});

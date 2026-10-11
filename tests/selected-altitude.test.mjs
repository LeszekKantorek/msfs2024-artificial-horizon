import test from 'node:test';
import assert from 'node:assert/strict';
import { initialAlert, advanceAlert, alertPresentation, selectedBugPosition } from '../web/altitude/selected.js';

function start(altitude = 8500, target = 10000) { return advanceAlert(initialAlert(), altitude, target, 0); }

test('both approaches cross inclusive thresholds once, using unrounded feet', () => {
  for (const sign of [-1, 1]) {
    let state = start(10000 + sign * 1500);
    state = advanceAlert(state, 10000 + sign * 1000.01, 10000, 10);
    assert.equal(state.flashStarted, null);
    state = advanceAlert(state, 10000 + sign * 1000, 10000, 20);
    assert.equal(state.kind, 'within-1000'); assert.equal(state.flashStarted, 20);
    for (const error of [1000, 1001, 999, 500, 200.01]) state = advanceAlert(state, 10000 + sign * error, 10000, 40);
    assert.equal(state.flashStarted, 20);
    state = advanceAlert(state, 10000 + sign * 200, 10000, 100);
    assert.equal(state.kind, 'within-200'); assert.equal(state.flashStarted, 100);
    state = advanceAlert(state, 10000 + sign * 10, 10000, 4000);
    assert.equal(state.flashStarted, 100);
  }
});

test('first sample and target changes establish a baseline without flashing', () => {
  for (const error of [1500, 1000, 500, 200, 0]) {
    const state = start(10000 - error);
    assert.equal(state.flashStarted, null);
    assert.equal(state.approached, error <= 1000); assert.equal(state.captured, error <= 200);
    const changed = advanceAlert(state, 10000 - error, 10000 - error, 1000);
    assert.equal(changed.flashStarted, null); assert.equal(changed.deviation, false);
  }
});

test('skipping both thresholds starts only the 200-foot alert', () => {
  const state = advanceAlert(start(), 9999, 10000, 500);
  assert.equal(state.kind, 'within-200'); assert.equal(state.approached, true);
  assert.equal(state.captured, true);
});

test('deviation persists after flashing, clears on return and alerts on the next departure', () => {
  for (const sign of [-1, 1]) {
    let state = start(10000);
    state = advanceAlert(state, 10000 + sign * 200, 10000, 10);
    assert.equal(state.deviation, false);
    state = advanceAlert(state, 10000 + sign * 200.01, 10000, 20);
    assert.equal(state.deviation, true); assert.equal(state.kind, 'deviation');
    state = advanceAlert(state, 10000 + sign * 500, 10000, 40);
    assert.equal(state.flashStarted, 20);
    assert.deepEqual(alertPresentation(state, 5020), { deviation: true, flashing: false, visible: true, nextUpdateAt: null });
    state = advanceAlert(state, 10000 + sign * 200, 10000, 5100);
    assert.equal(state.deviation, false); assert.equal(state.flashStarted, null);
    state = advanceAlert(state, 10000 - sign * 201, 10000, 5200);
    assert.equal(state.deviation, true); assert.equal(state.flashStarted, 5200);
  }
});

test('flash alternates at 250 ms and stops exactly at 5000 ms', () => {
  const state = advanceAlert(start(), 9000, 10000, 123);
  for (const [elapsed, visible, next] of [[0, true, 250], [249, true, 250], [250, false, 500], [499, false, 500], [500, true, 750], [4999, false, 5000]]) {
    const presentation = alertPresentation(state, 123 + elapsed);
    assert.equal(presentation.visible, visible); assert.equal(presentation.flashing, true);
    assert.equal(presentation.nextUpdateAt, 123 + next);
  }
  assert.deepEqual(alertPresentation(state, 5123), { deviation: false, flashing: false, visible: true, nextUpdateAt: null });
});

test('invalid or overflowing inputs reset history, with silent recovery including negative targets', () => {
  const active = advanceAlert(start(), 9000, 10000, 100);
  for (const invalid of [null, undefined, '10000', NaN, Infinity, -10000, 100000]) {
    for (const [altitude, target] of [[invalid, 10000], [9000, invalid]]) {
      const reset = advanceAlert(active, altitude, target, 200);
      assert.deepEqual(reset, initialAlert());
      const resumed = advanceAlert(reset, -120, -120, 500);
      assert.equal(resumed.flashStarted, null); assert.equal(resumed.captured, true);
    }
  }
});

test('bug uses the fixed tape spacing and pins the entire painted symbol at either edge', () => {
  assert.equal(selectedBugPosition(1000, 1100, 160), 40);
  assert.equal(selectedBugPosition(1000, 900, 160), 120);
  assert.equal(selectedBugPosition(1000, 5000, 160), 6.5);
  assert.equal(selectedBugPosition(1000, -120, 160), 153.5);
  assert.equal(selectedBugPosition(1000, 1100, 400), 160);
});

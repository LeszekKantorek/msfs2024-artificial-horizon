import test from 'node:test';
import assert from 'node:assert/strict';
import { verticalSpeedReading, vsiWindow } from '../web/altitude/vsi.js';
import { altitudeTicks, tapeMarkPresentation } from '../web/altitude/tape.js';
import { pfdLayout } from '../web/layout.js';

test('signed rates round to tens symmetrically and raw limits distinguish overflow from unavailable input', () => {
  for (const fpm of [undefined, null, '0', false, {}, [], NaN, Infinity, -Infinity]) {
    assert.deepEqual(verticalSpeedReading(fpm), { state: 'unavailable', text: 'X' });
  }
  for (const [fpm, text, state] of [
    [0, '0', 'available'], [-0, '0', 'available'], [-0.1, '0', 'available'], [0.1, '0', 'available'],
    [1, '0', 'available'], [-1, '0', 'available'], [4.999, '0', 'available'], [-4.999, '0', 'available'],
    [5, '+10', 'available'], [-5, '-10', 'available'], [12, '+10', 'available'], [-12, '-10', 'available'],
    [14.999, '+10', 'available'], [-14.999, '-10', 'available'], [15, '+20', 'available'], [-15, '-20', 'available'],
    [-20, '-20', 'available'], [-120, '-120', 'available'], [400, '+400', 'available'], [420, '+420', 'available'],
    [813.56, '+810', 'available'], [-813.56, '-810', 'available'],
    [1500, '+1500', 'available'], [-1500, '-1500', 'available'],
    [2000, '+2000', 'available'], [-2000, '-2000', 'available'],
    [2000.01, '+2000', 'available'], [-2000.01, '-2000', 'available'],
    [4881.36, '+4880', 'available'], [9995, '+10000', 'available'], [-9995, '-10000', 'available'],
    [9999, '+10000', 'available'], [-9999, '-10000', 'available'],
    [9999.01, '+OVR', 'overflow'], [-9999.01, '-OVR', 'overflow'],
    [Number.MAX_VALUE, '+OVR', 'overflow'], [-Number.MAX_VALUE, '-OVR', 'overflow'],
  ]) assert.deepEqual(verticalSpeedReading(fpm), { text, state });
});

test('VSI stays narrower than altitude with aligned right borders and a three-pixel painted gap', () => {
  for (const [width, height] of [[314, 234], [314, 474], [562, 234], [838, 284]]) {
    const layout = pfdLayout(width, height);
    const altitude = layout.altitude;
    const window = vsiWindow(altitude);
    assert.deepEqual([window.width, window.height], [51, 27]);
    assert.ok(window.width < altitude.width);
    assert.equal(window.x + window.width, altitude.width - 1);
    assert.equal(window.y - 0.5 - (altitude.height / 2 + 14 + 0.5), 3);
    assert.ok(window.y + window.height + 0.5 < altitude.height);
    assert.ok(window.x - 0.5 >= 7.5, 'VSI starts outside the painted seven-pixel target bug');
    assert.equal(altitude.x + altitude.width, width);
    assert.equal(layout.attitude.x + layout.attitude.width, altitude.x);
    assert.equal(layout.attitude.width, width - layout.airspeed.width - altitude.width);
    const oldHorizonWidth = width - layout.airspeed.width - altitude.width - Math.max(42, Math.min(54, width * 0.09));
    assert.ok(layout.attitude.width - oldHorizonWidth >= 42);
  }
});

test('whole labels disappear only around VSI and reappear as altitude moves; ticks keep left-side clearance', () => {
  const window = vsiWindow({ width: 60, height: 160 });
  const captured = altitudeTicks(10060, 160).find(row => row.value === 10000);
  assert.deepEqual(tapeMarkPresentation(captured, window), { tickEnd: 6, labelVisible: false });
  const departed = altitudeTicks(10160, 160).find(row => row.value === 10000);
  assert.deepEqual(tapeMarkPresentation(departed, window), { tickEnd: 12, labelVisible: true });
  for (const y of [window.y - 9, window.y + window.height + 9]) {
    assert.equal(tapeMarkPresentation({ major: true, y }, window).labelVisible, false);
  }
  for (const y of [window.y - 10, window.y + window.height + 10]) {
    assert.equal(tapeMarkPresentation({ major: true, y }, window).labelVisible, true);
  }
  assert.deepEqual(tapeMarkPresentation({ major: false, y: 104 }, window), { tickEnd: 6, labelVisible: false });
  assert.deepEqual(tapeMarkPresentation(captured, vsiWindow({ width: 92, height: 160 })), { tickEnd: 12, labelVisible: false });
});

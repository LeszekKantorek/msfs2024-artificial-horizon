import test from 'node:test';
import assert from 'node:assert/strict';
import { verticalSpeedReading, verticalSpeedPosition } from '../web/vertical-speed.js';

test('positive 1500 fpm is above the center and negative is symmetrically below it', () => {
  assert.equal(verticalSpeedPosition(1500, 160), 26);
  assert.equal(verticalSpeedPosition(-1500, 160), 134);
  assert.equal(verticalSpeedPosition(0, 160), 80);
});

test('the fixed range pins the whole pointer at inclusive boundaries and extreme finite values', () => {
  for (const height of [124, 160, 400]) {
    for (const fpm of [2000, 2000.01, 9999, Number.MAX_VALUE]) {
      assert.equal(verticalSpeedPosition(fpm, height), 8);
      assert.equal(verticalSpeedPosition(-fpm, height), height - 8);
    }
    assert.equal(verticalSpeedPosition(1000, height), height / 4 + 4);
  }
});

test('raw boundaries distinguish scale overflow, numeric overflow, and unavailable input', () => {
  for (const fpm of [undefined, null, '0', false, {}, [], NaN, Infinity, -Infinity]) {
    assert.deepEqual(verticalSpeedReading(fpm), { state: 'unavailable', text: 'X', offScale: false });
  }
  for (const [fpm, text, state, offScale] of [
    [0, '0', 'available', false], [-0, '0', 'available', false], [-0.1, '0', 'available', false],
    [0.1, '0', 'available', false], [813.56, '+814', 'available', false], [-813.56, '-814', 'available', false],
    [2000, '+2000', 'available', false], [-2000, '-2000', 'available', false],
    [2000.01, '+2000', 'available', true], [-2000.01, '-2000', 'available', true],
    [4881.36, '+4881', 'available', true], [9999, '+9999', 'available', true], [-9999, '-9999', 'available', true],
    [9999.01, '+OVR', 'overflow', true], [-9999.01, '-OVR', 'overflow', true],
    [Number.MAX_VALUE, '+OVR', 'overflow', true], [-Number.MAX_VALUE, '-OVR', 'overflow', true],
  ]) assert.deepEqual(verticalSpeedReading(fpm), { text, state, offScale });
});

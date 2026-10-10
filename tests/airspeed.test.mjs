import test from 'node:test';
import assert from 'node:assert/strict';
import { speedState, rollingDigits, tapeOffset } from '../web/airspeed.js';

test('speed availability distinguishes invalid data, zero and display overflow without clamping', () => {
  for (const value of [undefined, null, false, '0', {}, [], NaN, Infinity, -Infinity, -0.01])
    assert.equal(speedState(value), 'unavailable');
  for (const value of [0, 9.5, 99.5, 999]) assert.equal(speedState(value), 'available');
  for (const value of [999.01, 1000, Number.MAX_VALUE]) assert.equal(speedState(value), 'overflow');
});

test('rolling columns carry together and suppress leading zeros in both directions', () => {
  const rows = [
    [0, ['', '', '0'], ['', '', '1'], [0, 0, 0]],
    [9.5, ['', '', '9'], ['', '1', '0'], [0, 0.5, 0.5]],
    [10, ['', '1', '0'], ['', '2', '1'], [0, 0, 0]],
    [99.5, ['', '9', '9'], ['1', '0', '0'], [0.5, 0.5, 0.5]],
    [100, ['1', '0', '0'], ['2', '1', '1'], [0, 0, 0]],
    [109.5, ['1', '0', '9'], ['2', '1', '0'], [0, 0.5, 0.5]],
    [999, ['9', '9', '9'], ['0', '0', '0'], [0, 0, 0]],
  ];
  for (const [value, current, next, offset] of [...rows, ...rows.toReversed()]) {
    const digits = rollingDigits(value);
    assert.deepEqual(digits.map(d => d.current), current);
    assert.deepEqual(digits.map(d => d.next), next);
    assert.deepEqual(digits.map(d => d.offset), offset);
  }
});

test('independent tape positions place higher speeds above the pointer and move down on acceleration', () => {
  for (const [speed, tick, expected] of [[0, 0, 0], [0, 10, -40], [99.5, 100, -2],
    [100, 100, 0], [100.5, 100, 2], [999, 990, 36]]) {
    assert.equal(tapeOffset(speed, tick), expected);
  }
});

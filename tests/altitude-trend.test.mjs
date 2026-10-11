import test from 'node:test';
import assert from 'node:assert/strict';
import { altitudeTrend } from '../web/altitude/trend.js';

test('source climb and descent predict six seconds on the fixed altitude tape', () => {
  assert.deepEqual(altitudeTrend(10060, 1500, 400), { prediction: 10210, y: 140 });
  assert.deepEqual(altitudeTrend(10060, -1500, 400), { prediction: 9910, y: 260 });
  assert.deepEqual(altitudeTrend(-120, 1500, 160), { prediction: 30, y: 20 });
  assert.deepEqual(altitudeTrend(10060, 1500, 160), { prediction: 10210, y: 20 });
});

test('trend requires displayable altitude and a finite nonzero source rate', () => {
  for (const invalid of [undefined, null, '1500', false, NaN, Infinity, -Infinity, 0, -0]) {
    assert.equal(altitudeTrend(10000, invalid, 160), null);
  }
  for (const invalid of [undefined, null, '10000', NaN, Infinity, -10000, 100000]) {
    assert.equal(altitudeTrend(invalid, 1500, 160), null);
  }
  assert.notEqual(altitudeTrend(-9999, -1, 160), null);
  assert.notEqual(altitudeTrend(99999, 1, 160), null);
});

test('finite rates outside the numeric VSI range clip the trend without losing direction', () => {
  for (const fpm of [9999, 10000, Number.MAX_VALUE]) {
    const climb = altitudeTrend(10000, fpm, 160), descent = altitudeTrend(10000, -fpm, 160);
    assert.equal(climb.y, 1); assert.equal(descent.y, 159);
    assert.ok(Number.isFinite(climb.prediction) && Number.isFinite(descent.prediction));
  }
  assert.equal(altitudeTrend(10000, 2000, 160).y, 1);
  assert.equal(altitudeTrend(10000, -2000, 160).y, 159);
});

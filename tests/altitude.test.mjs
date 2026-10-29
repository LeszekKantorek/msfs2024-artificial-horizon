import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { altitudeState, rollingAltitude } from '../web/altitude/readout.js';
import { altitudeTicks, tickPosition } from '../web/altitude/tape.js';
import { baroReading, numericBaroReading } from '../web/altitude/baro.js';
import { normalizeOptionalIndications } from '../web/telemetry/validation.js';

test('signed altitude remains independently valid beyond display limits', () => {
  for (const bad of [undefined, null, false, '0', {}, [], NaN, Infinity, -Infinity]) {
    assert.equal(altitudeState(bad), 'unavailable');
    assert.equal(normalizeOptionalIndications({ altitude_ft: bad, baro_text: '29.92 inHg' }).altitude_ft, null);
  }
  for (const value of [-9999, -0.1, -0, 0, 99999]) assert.equal(altitudeState(value), 'available');
  for (const value of [-10000, 99999.01, Number.MAX_VALUE]) {
    assert.equal(altitudeState(value), 'overflow');
    assert.equal(normalizeOptionalIndications({ altitude_ft: value }).altitude_ft, value);
  }
  for (const row of JSON.parse(readFileSync(new URL('./fixtures/altitude-samples.json', import.meta.url)))) {
    const sample = normalizeOptionalIndications(row);
    assert.equal(sample.altitude_ft, row.altitude_ft);
    assert.equal(sample.baro_text, row.baro_text);
  }
});

test('BARO forwards every string unchanged and rejects only non-string data', () => {
  for (const text of ['23.83 inHg', '1026 hPa', 'BARO', '', '  BARO  ', '<b>BARO</b>', 'OVR', 'X']) {
    const sample = normalizeOptionalIndications({ altitude_ft: -120, baro_text: text });
    assert.equal(sample.altitude_ft, -120);
    assert.equal(sample.baro_text, text);
    assert.deepEqual(baroReading(sample.baro_text), { state: 'available', text });
  }
  for (const value of [undefined, null, false, {}, [], 0, 29.92, NaN, Infinity]) {
    const sample = normalizeOptionalIndications({ altitude_ft: -120, baro_text: value });
    assert.equal(sample.altitude_ft, -120);
    assert.equal(sample.baro_text, null);
    assert.deepEqual(baroReading(sample.baro_text), { state: 'unavailable', text: 'X' });
  }
});

test('numeric BARO validates presence, converts units and uses explicit STD', () => {
  const normal = { pressure_hpa: 1013.25, standard: false };
  assert.deepEqual(numericBaroReading(normal, 'inHg'), { state: 'available', text: '29.92 inHg' });
  assert.deepEqual(numericBaroReading(normal, 'hPa'), { state: 'available', text: '1013 hPa' });
  assert.deepEqual(numericBaroReading({ ...normal, standard: true }, 'hPa'), { state: 'available', text: 'STD' });
  assert.equal(Object.hasOwn(normalizeOptionalIndications({ baro_text: '' }), 'baro_setting'), false);
  for (const bad of [null, {}, { pressure_hpa: 1013 }, { pressure_hpa: 0, standard: false },
    { pressure_hpa: Infinity, standard: false }, { pressure_hpa: 1013, standard: 0 }]) {
    const result = normalizeOptionalIndications({ baro_text: 'old', baro_setting: bad });
    assert.equal(Object.hasOwn(result, 'baro_setting'), true);
    assert.equal(result.baro_setting, null);
    assert.equal(numericBaroReading(result.baro_setting, 'hPa').text, 'X');
  }
  assert.equal(numericBaroReading({ pressure_hpa: Number.MAX_VALUE, standard: false }, 'hPa').text, 'X');
});

test('altitude tape keeps fixed pixel spacing with independent tick positions and signed labels', () => {
  for (const [tick, expected] of [[1200, 0], [1100, 40], [1000, 80], [900, 120], [800, 160]])
    assert.equal(tickPosition(1000, tick, 160), expected);
  assert.equal(tickPosition(1010, 1000, 160), 84);
  assert.equal(tickPosition(990, 1000, 160), 76);
  assert.equal(tickPosition(-100, -200, 160), 120);
  assert.equal(tickPosition(1000, 1100, 320), 120);
  const ticks = altitudeTicks(-100, 160);
  assert.equal(ticks.length, 23);
  assert.equal(ticks[0].value, -320);
  assert.deepEqual(ticks.filter(t => t.major).map(t => t.value), [-300, -200, -100, 0, 100]);
  assert.equal(ticks.find(t => t.value === 0).y, 40);
});

test('taller altitude tapes show more ticks and labels without stretching their spacing', () => {
  for (const [height, low, high, visibleCount, majorCount] of [
    [160, 800, 1200, 21, 5], [400, 500, 1500, 51, 11], [1200, -500, 2500, 151, 31],
  ]) {
    const ticks = altitudeTicks(1000, height);
    const visible = ticks.filter(t => t.y >= 0 && t.y <= height);
    assert.equal(visible.length, visibleCount);
    assert.equal(visible.filter(t => t.major).length, majorCount);
    assert.equal(visible[0].value, low);
    assert.equal(visible.at(-1).value, high);
    assert.equal(visible[0].y, height);
    assert.equal(visible.at(-1).y, 0);
    for (let i = 1; i < ticks.length; i++) assert.equal(ticks[i - 1].y - ticks[i].y, 8);
  }
  const fractional = altitudeTicks(-0.5, 401);
  assert.ok(fractional[0].y > 401 && fractional.at(-1).y < 0);
  assert.equal(fractional.find(t => t.value === 0).y, 200.3);
});

test('20-foot drums carry higher columns, suppress leading zeros and reverse without state', () => {
  const rows = [
    [0, ['', '', '', '00'], ['', '', '', '20'], [0, 0, 0, 0]],
    [10, ['', '', '', '00'], ['', '', '', '20'], [0, 0, 0, 0.5]],
    [20, ['', '', '', '20'], ['', '', '', '40'], [0, 0, 0, 0]],
    [90, ['', '', '', '80'], ['', '', '1', '00'], [0, 0, 0.5, 0.5]],
    [100, ['', '', '1', '00'], ['', '', '1', '20'], [0, 0, 0, 0]],
    [990, ['', '', '9', '80'], ['', '1', '0', '00'], [0, 0.5, 0.5, 0.5]],
    [1000, ['', '1', '0', '00'], ['', '1', '0', '20'], [0, 0, 0, 0]],
    [9990, ['', '9', '9', '80'], ['1', '0', '0', '00'], [0.5, 0.5, 0.5, 0.5]],
    [10000, ['1', '0', '0', '00'], ['1', '0', '0', '20'], [0, 0, 0, 0]],
  ];
  for (const [value, current, next, offset] of [...rows, ...rows.toReversed()]) {
    const { columns, sign } = rollingAltitude(value);
    assert.equal(sign, '');
    assert.deepEqual(columns.map(d => d.current), current);
    assert.deepEqual(columns.map(d => d.next), next);
    assert.deepEqual(columns.map(d => d.offset), offset);
  }
  for (const value of [-20, -0.1, 0, 0.1, 20, 0.1, 0, -0.1, -20]) {
    const reading = rollingAltitude(value);
    assert.equal(reading.sign, value < 0 ? '−' : '');
    assert.deepEqual(reading.columns, rollingAltitude(Math.abs(value)).columns);
  }
  assert.equal(rollingAltitude(-0).sign, '');
  assert.equal(rollingAltitude(0.5).columns.at(-1).offset, 0.025);
});

test('both display boundaries leave the out-of-range next drum row blank', () => {
  for (const [value, current] of [[99999, ['9', '9', '9', '80']], [-9999, ['', '9', '9', '80']]]) {
    const { columns } = rollingAltitude(value);
    assert.deepEqual(columns.map(d => d.current), current);
    assert.deepEqual(columns.map(d => d.next), ['', '', '', '']);
    assert.ok(columns.every(d => d.offset === 0 || d.offset === 0.95));
  }
});

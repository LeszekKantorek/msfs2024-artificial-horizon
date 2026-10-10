import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { speedRanges, speedIndications, referenceReadout, placeReferenceLabels, rangeBounds } from '../web/airspeed/calculations.js';
import { validateAirspeedProfile, optionalIndications } from '../web/telemetry-client.js';

const profile = JSON.parse(readFileSync(new URL('./fixtures/airspeed-profile.json', import.meta.url)));

test('profile validation preserves known knots and rejects malformed or incoherent configuration locally', () => {
  assert.deepEqual(validateAirspeedProfile(profile), profile);
  for (const bad of [undefined, null, [], false, 'profile', {},
    { ...profile, vs0_kt: 0 }, { ...profile, vs1_kt: 39 },
    { ...profile, vfe_kt: 49 }, { ...profile, vno_kt: 85 },
    { ...profile, vne_kt: 110 }, { ...profile, references: null },
    { ...profile, references: { ...profile.references, vr_kt: '55' } },
    { ...profile, references: { ...profile.references, vy_kt: 0 } },
    { ...profile, references: { ...profile.references, vg_kt: 141 } }]) {
    assert.equal(validateAirspeedProfile(bad), null);
    const sample = optionalIndications({ airspeed_profile: bad, ias_kt: 123, gs_kt: 137, ias_acceleration_ktps: 2 });
    assert.equal(sample.airspeed_profile, null);
    assert.equal(sample.ias_kt, 123);
    assert.equal(sample.ias_acceleration_ktps, 2);
  }
  for (const key of ['vs0_kt', 'vs1_kt', 'vfe_kt', 'vno_kt', 'vne_kt']) {
    for (const bad of [undefined, null, false, '40', NaN, Infinity, -Infinity, -1])
      assert.equal(validateAirspeedProfile({ ...profile, [key]: bad }), null);
  }
  for (const key of ['vr_kt', 'vx_kt', 'vy_kt', 'vg_kt']) {
    for (const bad of [undefined, null, false, '55', NaN, Infinity, -Infinity, -1])
      assert.equal(validateAirspeedProfile({ ...profile, references: { ...profile.references, [key]: bad } }), null);
  }
  // Unknown additive fields are ignored, and normalized data does not share mutable input objects.
  const input = structuredClone(profile); input.extra = 123;
  const normalized = validateAirspeedProfile(input);
  input.references.vr_kt = 1;
  assert.deepEqual(normalized, profile);
});

test('five configured ranges include parallel flap and normal ranges at independent thresholds', () => {
  const ranges = speedRanges(profile);
  assert.deepEqual(ranges, [
    { name: 'low', min: 0, max: 40 }, { name: 'flaps', min: 40, max: 85 },
    { name: 'normal', min: 50, max: 110 }, { name: 'caution', min: 110, max: 140 },
    { name: 'overspeed', min: 140, max: Infinity },
  ]);
  for (const [speed, expected] of [
    [39.99, ['low']], [40, ['flaps']], [40.01, ['flaps']],
    [49.99, ['flaps']], [50, ['flaps', 'normal']], [50.01, ['flaps', 'normal']],
    [84.99, ['flaps', 'normal']], [85, ['normal']], [85.01, ['normal']],
    [109.99, ['normal']], [110, ['caution']], [110.01, ['caution']],
    [139.99, ['caution']], [140, ['overspeed']], [140.01, ['overspeed']],
  ]) assert.deepEqual(ranges.filter(r => r.min <= speed && speed < r.max).map(r => r.name), expected);
});

test('unbounded overspeed ranges clip to the tape while invalid coordinates remain invalid', () => {
  const tape = { centerY: 77, clipTopY: 1, clipBottomY: 137 };
  const overspeed = speedRanges(profile).find(range => range.name === 'overspeed');
  for (const [ias, expected] of [[110, { y: 1, height: 0 }],
    [140, { y: 1, height: 76 }], [150, { y: 1, height: 116 }], [999, { y: 1, height: 136 }]]) {
    assert.deepEqual(rangeBounds(ias, overspeed, tape), expected);
  }
  assert.deepEqual(rangeBounds(75, { min: 0, max: Infinity }, tape), { y: 1, height: 136 });
  assert.deepEqual(rangeBounds(75, { min: -Infinity, max: Infinity }, tape), { y: 1, height: 136 });
  for (const bounds of [rangeBounds(NaN, overspeed, tape),
    rangeBounds(140, overspeed, { ...tape, centerY: NaN })]) {
    assert.ok(Number.isNaN(bounds.y));
    assert.ok(Number.isNaN(bounds.height));
  }
});

test('source acceleration predicts six seconds and separates actual overspeed from approaching VNE', () => {
  for (const [ias, acceleration, prediction, overspeed, approaching] of [
    [100, 5, 130, false, false], [110, 5, 140, false, true], [110, 5.01, 140.06, false, true],
    [139.99, 0, null, false, false], [140, 0, null, true, false], [140.01, 0, null, true, false],
    [150, -25, 0, true, false], [10, -25, -140, false, false], [0, 25, 150, false, true],
  ]) assert.deepEqual(speedIndications(ias, acceleration, profile), { prediction, overspeed, approaching });
  for (const bad of [undefined, null, false, '5', {}, [], NaN, Infinity, -Infinity, Number.MAX_VALUE]) {
    assert.deepEqual(speedIndications(150, bad, profile), { prediction: null, overspeed: true, approaching: false });
  }
  for (const bad of [null, -1])
    assert.deepEqual(speedIndications(bad, 25, profile), { prediction: null, overspeed: false, approaching: false });
  assert.deepEqual(speedIndications(999.01, 25, profile), { prediction: null, overspeed: true, approaching: false });
  assert.deepEqual(speedIndications(110, 5, null), { prediction: 140, overspeed: false, approaching: false });
});

test('independent demo checkpoints produce the expected six-second predictions', () => {
  const speeds = JSON.parse(readFileSync(new URL('./fixtures/speed-samples.json', import.meta.url)));
  const indications = JSON.parse(readFileSync(new URL('./fixtures/speed-indications.json', import.meta.url)));
  for (const row of indications) {
    const speed = speeds.find(s => s.sample === row.sample);
    assert.equal(speedIndications(speed.ias_kt, row.ias_acceleration_ktps, profile).prediction, row.prediction_kt);
  }
});

test('fallback references stay compact and colliding labels retain their exact speed anchors', () => {
  for (const [value, expected] of [[55, '55'], [55.123456789, '55.1'], [0.01, '<0.1'],
    [1e-300, '<0.1'], [998.99, '999'], [999, '999'], [1234.5678, 'OVR'], [Number.MAX_VALUE, 'OVR']])
    assert.equal(referenceReadout(value), expected);
  const ordinary = placeReferenceLabels(75, [55, 65, 75, 90], { centerY: 200, clipTopY: 1, clipBottomY: 383 });
  assert.deepEqual(ordinary.map(p => p.labelPosition), [280, 240, 200, 140]);
  for (const values of [[60, 60, 60, 60], [60, 60.1, 60.2, 60.3]]) {
    for (const ias of [44, 60, 74]) {
      const positions = placeReferenceLabels(ias, values, { centerY: 77, clipTopY: 1, clipBottomY: 137 });
      const visible = positions.filter(p => p.visible).sort((a, b) => a.labelPosition - b.labelPosition);
      positions.forEach((p, i) => assert.equal(p.position, 77 + (ias - values[i]) * 4));
      assert.ok(visible.every(p => p.labelPosition >= 9 && p.labelPosition <= 129));
      assert.ok(visible.slice(1).every((p, i) => p.labelPosition - visible[i].labelPosition >= 16));
    }
  }
});

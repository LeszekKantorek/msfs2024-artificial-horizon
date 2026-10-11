import test from 'node:test';
import assert from 'node:assert/strict';

test('helpers, visual objects and telemetry import and calculate in Node without a document', async () => {
  assert.equal(typeof document, 'undefined');
  const helpers = await import('../web/helpers.js');
  const speed = await import('../web/airspeed/helpers.js');
  const readout = await import('../web/airspeed/readout.js');
  const tape = await import('../web/airspeed/tape.js');
  const references = await import('../web/airspeed/references.js');
  const ias = await import('../web/airspeed/ias.js');
  const telemetry = await import('../web/telemetry/client.js');
  const airspeed = await import('../web/airspeed/index.js');
  const horizon = await import('../web/horizon/index.js');
  const fixedSymbols = await import('../web/fixed-symbols/index.js');
  assert.deepEqual(Object.keys(helpers).sort(), ['clamp', 'setAttributes', 'svgElement']);
  assert.equal(helpers.clamp(-1, 0, 10), 0);
  assert.equal(helpers.clamp(5, 0, 10), 5);
  assert.equal(helpers.clamp(11, 0, 10), 10);
  assert.ok(Number.isNaN(helpers.clamp(NaN, 0, 10)));
  assert.equal(speed.speedState(999.01), 'overflow');
  assert.equal(speed.tapeOffset(99.5, 100), -2);
  assert.deepEqual(readout.rollingDigits(9.5).map(digit => digit.offset), [0, 0.5, 0.5]);
  assert.deepEqual(tape.rangeBounds(75, { min: 0, max: Infinity },
    { centerY: 77, clipTopY: 1, clipBottomY: 137 }), { y: 1, height: 136 });
  assert.equal(references.referenceReadout(0.01), '<0.1');
  assert.deepEqual(ias.speedIndications(110, 5, { vne_kt: 140 }),
    { prediction: 140, overspeed: false, approaching: true });
  assert.equal(typeof telemetry.createTelemetryClient, 'function');
  assert.deepEqual(Object.keys(airspeed), ['createAirspeed']);
  assert.equal(typeof horizon.createHorizon, 'function');
  assert.deepEqual(horizon.attitudeTransforms({ pitch_deg: 15, roll_deg: 30 }),
    { rotation: 'rotate(-30 0 0)', translation: 'translate(0 60)' });
  assert.deepEqual(Object.keys(fixedSymbols), ['createFixedSymbols']);
});

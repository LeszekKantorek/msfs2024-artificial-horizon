import test from 'node:test';
import assert from 'node:assert/strict';

test('helpers, calculations and telemetry import in Node without a document', async () => {
  assert.equal(typeof document, 'undefined');
  const helpers = await import('../web/helpers.js');
  const calculations = await import('../web/airspeed/calculations.js');
  const telemetry = await import('../web/telemetry/client.js');
  const airspeed = await import('../web/airspeed/index.js');
  const horizonCalculations = await import('../web/horizon/calculations.js');
  const horizon = await import('../web/horizon/index.js');
  const fixedSymbols = await import('../web/fixed-symbols/index.js');
  assert.deepEqual(Object.keys(helpers).sort(), ['clamp', 'setAttributes', 'svgElement']);
  assert.equal(helpers.clamp(-1, 0, 10), 0);
  assert.equal(helpers.clamp(5, 0, 10), 5);
  assert.equal(helpers.clamp(11, 0, 10), 10);
  assert.ok(Number.isNaN(helpers.clamp(NaN, 0, 10)));
  assert.equal(typeof calculations.deriveSpeedReadings, 'function');
  assert.equal(typeof telemetry.createTelemetryClient, 'function');
  assert.deepEqual(Object.keys(airspeed), ['createAirspeed']);
  assert.equal(typeof horizonCalculations.attitudeTransforms, 'function');
  assert.deepEqual(Object.keys(horizon), ['createHorizon']);
  assert.deepEqual(Object.keys(fixedSymbols), ['createFixedSymbols']);
});

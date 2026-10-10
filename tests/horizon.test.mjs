import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { attitudeTransforms } from '../web/horizon.js';

// Expected transforms are independent of the implementation and reuse source poses.
test('known poses use the correct signs and local pitch displacement', () => {
  const poses = JSON.parse(readFileSync(new URL('./fixtures/attitudes.json', import.meta.url)));
  const expected = [[0, 0], [0, 60], [0, -60], [30, 0], [-30, 0], [-25, 40], [25, -40]];
  for (const [index, pose] of poses.entries()) {
    const [rotation, offset] = expected[index];
    assert.deepEqual(attitudeTransforms(pose), {
      rotation: `rotate(${rotation} 0 0)`, translation: `translate(0 ${offset})`,
    });
  }
  for (const [pitch, roll, rotation, offset] of [[90, -180, 180, 360], [-90, 179.999, -179.999, -360]]) {
    assert.deepEqual(attitudeTransforms({ pitch_deg: pitch, roll_deg: roll }), {
      rotation: `rotate(${rotation} 0 0)`, translation: `translate(0 ${offset})`,
    });
  }
});

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

test('world markup translates pitch inside bank rotation, not in screen coordinates', () => {
  const html = readFileSync(new URL('../web/index.html', import.meta.url), 'utf8');
  assert.match(html, /id="world-rotation"><g id="world-pitch">/);
  const transforms = attitudeTransforms({ pitch_deg: 10, roll_deg: 25 });
  // Interpret the nested SVG operations at the horizon's centre. Independent expected
  // coordinates catch a sign error; the hierarchy assertion catches swapped operations.
  const angle = Number(transforms.rotation.match(/rotate\((\S+)/)[1]) * Math.PI / 180;
  const dy = Number(transforms.translation.match(/translate\(0 (\S+)\)/)[1]);
  const x = - Math.sin(angle) * dy;
  const y = Math.cos(angle) * dy;
  assert.ok(Math.abs(x - 16.90473047) < 1e-7);
  assert.ok(Math.abs(y - 36.25231148) < 1e-7);
});

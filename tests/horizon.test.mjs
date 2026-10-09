import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { attitudeTransforms, createHorizonRenderer } from '../web/horizon.js';
import { pfdLayout } from '../web/layout.js';

// Expected transforms are independent of the implementation and reuse source poses.
test('known poses use the correct signs and local pitch displacement', () => {
  const poses = JSON.parse(readFileSync(new URL('./fixtures/attitudes.json', import.meta.url)));
  const expected = [[0, 0], [0, 60], [0, -60], [30, 0], [-30, 0], [-25, 40], [25, -40]];
  for (const [index, pose] of poses.entries()) {
    const [rotation, offset] = expected[index];
    assert.deepEqual(attitudeTransforms(pose), {
      rotation: `rotate(${rotation} 160 160)`, translation: `translate(0 ${offset})`,
    });
  }
  for (const [pitch, roll, rotation, offset] of [[90, -180, 180, 360], [-90, 179.999, -179.999, -360]]) {
    assert.deepEqual(attitudeTransforms({ pitch_deg: pitch, roll_deg: roll }), {
      rotation: `rotate(${rotation} 160 160)`, translation: `translate(0 ${offset})`,
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
  const x = 160 - Math.sin(angle) * dy;
  const y = 160 + Math.cos(angle) * dy;
  assert.ok(Math.abs(x - 176.90473047) < 1e-7);
  assert.ok(Math.abs(y - 196.25231148) < 1e-7);
});

function harness() {
  let time = 0;
  let nextId = 0;
  const frames = new Map();
  const drawn = [];
  const obscured = [];
  const renderer = createHorizonRenderer({
    draw: value => drawn.push(value), obscure: reason => obscured.push(reason),
    now: () => time,
    requestFrame: fn => { const id = ++nextId; frames.set(id, fn); return id; },
    cancelFrame: id => frames.delete(id),
  });
  return { renderer, frames, drawn, obscured,
    advance: ms => { time += ms; },
    flush() { const pending = [...frames.values()]; frames.clear(); for (const fn of pending) fn(); },
  };
}
const sample = (pitch = 0, roll = 0, expiresAt = 1000) => ({
  attitude: { pitch_deg: pitch, roll_deg: roll }, expiresAt,
});

test('a burst draws only its newest sample in one frame, without continuous animation', () => {
  const h = harness();
  h.renderer.accept(sample(15)); h.renderer.accept(sample(-15)); h.renderer.accept(sample(10, 25));
  assert.equal(h.frames.size, 1); assert.equal(h.drawn.length, 0);
  h.flush();
  assert.deepEqual(h.drawn, [{ rotation: 'rotate(-25 160 160)', translation: 'translate(0 40)' }]);
  assert.equal(h.frames.size, 0);
  h.renderer.accept(sample()); h.flush(); assert.equal(h.drawn.length, 2);
});

test('a sample expiring before its frame is obscured even if timeout delivery is delayed', () => {
  const h = harness(); h.renderer.accept(sample(15, 30, 500));
  h.advance(500); h.flush();
  assert.equal(h.drawn.length, 0); assert.deepEqual(h.obscured, ['stale']);
  h.renderer.accept(sample(0, 0, 1500)); h.flush(); assert.equal(h.drawn.length, 1);
});

test('unavailable and suspended states cancel pending rendering and discard the old sample', () => {
  for (const reason of ['connecting', 'reconnecting', 'suspended', 'waiting', 'paused', 'stale', 'disconnected', 'invalid']) {
    const h = harness(); h.renderer.accept(sample(10, 25));
    const delayed = [...h.frames.values()][0];
    h.renderer.invalidate(reason); h.flush(); delayed();
    assert.equal(h.frames.size, 0); assert.equal(h.drawn.length, 0);
    assert.deepEqual(h.obscured, [reason]);
    h.renderer.accept(sample(-10, -25)); h.flush(); assert.equal(h.drawn.length, 1);
  }
});

test('resize preserves the last values and deadline, coalescing with new samples', () => {
  const h = harness();
  h.renderer.accept(sample(10, 25, 500)); h.flush();
  const portrait = pfdLayout(320, 440);
  h.renderer.resize(portrait); h.renderer.resize(pfdLayout(568, 204));
  assert.equal(h.frames.size, 1);
  h.flush();
  assert.equal(h.drawn.length, 2);
  const last = h.drawn.at(-1);
  assert.match(last.rotation, /^rotate\(-25 /);
  assert.equal(h.frames.size, 0);
  h.advance(500);
  h.renderer.resize(portrait); h.flush();
  assert.equal(h.drawn.length, 2);
  assert.deepEqual(h.obscured, ['stale']);
  h.renderer.resize(portrait); h.flush();
  assert.equal(h.frames.size, 0);
});

test('an already cancelled frame cannot draw a newer subscription sample', () => {
  const h = harness(); h.renderer.accept(sample(10, 25));
  const oldCallback = [...h.frames.values()][0];
  h.renderer.invalidate('reconnecting');
  h.renderer.accept(sample(-10, -25));
  oldCallback();
  assert.equal(h.drawn.length, 0);
  assert.equal(h.frames.size, 1);
  h.flush(); assert.equal(h.drawn.length, 1);
});

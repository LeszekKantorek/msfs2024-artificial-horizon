import test from 'node:test';
import assert from 'node:assert/strict';
import { pfdLayout } from '../web/layout.js';

test('portrait and short landscape allocate disjoint instrument regions without changing symbol proportions', () => {
  for (const [width, height] of [[314, 444], [384, 628], [562, 204], [661, 244], [838, 284], [314, 204]]) {
    const l = pfdLayout(width, height);
    assert.ok(l.attitude.width >= 160);
    assert.ok(l.attitude.height >= 120);
    const regions = [l.airspeed, l.attitude, l.altitude];
    for (const r of [...regions, l.heading]) {
      assert.ok(r.x >= 0 && r.y >= 0 && r.width > 0 && r.height > 0);
      assert.ok(r.x + r.width <= width && r.y + r.height <= height);
    }
    for (let i = 1; i < regions.length; i++) {
      assert.ok(regions[i - 1].x + regions[i - 1].width <= regions[i].x);
    }
    assert.ok(l.heading.y + l.heading.height <= l.attitude.y);
    assert.ok(l.slipY > l.attitude.y + l.attitude.height);
    assert.ok(l.turnY > l.slipY && l.turnY + 5 < height);
    assert.ok(l.labelSize >= 12);
    assert.ok(l.pitchScale * 2.5 >= 8);
    assert.ok(l.aircraftWidth < l.attitude.width);
    // The zero triangle extends 18px above the radius; ticks extend 12px.
    assert.ok(l.cy - l.bankRadius - 19 >= l.attitude.y);
    assert.ok((l.bankRadius + 13) * Math.sin(Math.PI / 3) <= l.attitude.width / 2);
  }
});

test('altitude retains its tape width and returns the outside trend space to the horizon', () => {
  for (const [width, height, tapeWidth] of [[320, 240, 60.8], [320, 480, 60.8], [568, 240, 92], [844, 320, 92]]) {
    const layout = pfdLayout(width, height);
    assert.equal(layout.altitude.width, tapeWidth);
    assert.equal(layout.altitude.x + tapeWidth, width);
    assert.equal(layout.attitude.x + layout.attitude.width, layout.altitude.x);
    assert.equal(layout.heading.width, layout.attitude.width);
    assert.equal(layout.altitude.height, height - 80);
    assert.equal(layout.attitude.width, width - layout.airspeed.width - tapeWidth);
  }
});

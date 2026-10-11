import test from 'node:test';
import assert from 'node:assert/strict';
import { headingText, validHeading, headingDifference, headingBugPosition, headingTicks } from '../web/heading.js';

test('north wrap puts a two-degree selected heading eight pixels right, with continuous tape motion', () => {
  assert.equal(headingBugPosition(359, 1, 200), 108);
  assert.equal(headingBugPosition(1, 359, 200), 92);
  const right = headingTicks(359, 200).find(row => row.value === 10);
  const north = headingTicks(0, 200).find(row => row.value === 10);
  assert.equal(north.x - right.x, -4);
  assert.equal(headingTicks(1, 200).find(row => row.value === 350).x - headingTicks(0, 200).find(row => row.value === 350).x, -4);
});

test('three-digit magnetic headings use 360 for rounded north and reject invalid raw values', () => {
  for (const [degrees, expected] of [[0, '360'], [-0, '360'], [0.49, '360'], [0.5, '001'], [5, '005'], [89.5, '090'], [359.49, '359'], [359.5, '360'], [359.999, '360']]) {
    assert.equal(headingText(degrees), expected); assert.equal(validHeading(degrees), true);
  }
  for (const bad of [null, undefined, '0', false, {}, [], NaN, Infinity, -Infinity, -0.001, 360, 721]) {
    assert.equal(validHeading(bad), false); assert.equal(headingText(bad), 'X');
    assert.equal(headingBugPosition(bad, 0, 200), null);
    assert.equal(headingBugPosition(0, bad, 200), null);
    assert.deepEqual(headingTicks(bad, 200), []);
  }
});

test('shortest differences choose left at 180 degrees and pin the complete painted bug', () => {
  assert.equal(headingDifference(0, 180), -180); assert.equal(headingDifference(180, 0), -180);
  assert.equal(headingDifference(350, 10), 20); assert.equal(headingDifference(10, 350), -20);
  for (const width of [190, 300, 662]) {
    assert.equal(headingBugPosition(0, 0, width), width / 2);
    assert.equal(headingBugPosition(0, 180, width), 8.5);
    assert.equal(headingBugPosition(0, 179, width), width - 8.5);
  }
});

test('degree ticks keep fixed spacing and hide whole labels near the window and either clip edge', () => {
  for (const width of [190, 300, 662]) {
    const rows = headingTicks(359.6, width);
    for (let index = 1; index < rows.length; index++) assert.ok(Math.abs(rows[index].x - rows[index - 1].x - 4) < 1e-10);
    for (const row of rows) {
      assert.equal(row.major, row.value % 5 === 0);
      if (row.labelVisible) {
        assert.equal(row.value % 10, 0);
        assert.ok(row.x - 11 >= 0 && row.x + 11 <= width);
        assert.ok(Math.abs(row.x - width / 2) > 34);
      }
    }
    assert.ok(rows.some(row => row.labelVisible));
    assert.equal(rows.find(row => row.value === 0).labelVisible, false);
  }
  assert.ok(headingTicks(359.6, 662).filter(row => row.labelVisible).length > headingTicks(359.6, 190).filter(row => row.labelVisible).length);
});

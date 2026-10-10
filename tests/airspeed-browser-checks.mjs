import assert from 'node:assert/strict';

// Uses actual SVG paint and the fixture SSE path, with independent expected values.
export async function checkAirspeedIndications(page, setFixture, profile) {
  for (const [width, height] of [[320, 240], [320, 480], [568, 240]]) {
    await page.setViewportSize({ width, height });
    for (const [ias, acceleration, prediction, overspeed, approaching] of [
      [0, 25, 150, false, true], [39.99, 0, null, false, false],
      [40, 0, null, false, false], [40.01, 0, null, false, false],
      [49.99, 0, null, false, false], [50, 0, null, false, false], [50.01, 0, null, false, false],
      [84.99, 0, null, false, false], [85, 0, null, false, false], [85.01, 0, null, false, false],
      [109.99, 0, null, false, false], [110, 5, 140, false, true], [110.01, 5, 140.01, false, true],
      [139.99, 0, null, false, false], [140, 0, null, true, false], [140.01, 0, null, true, false],
      [150, -25, 0, true, false], [10, -25, -140, false, false], [999, 0, null, true, false],
    ]) {
      setFixture({ ias_kt: ias, gs_kt: 137, airspeed_profile: profile, ias_acceleration_ktps: acceleration });
      await page.waitForFunction(({ ias, prediction, overspeed, approaching }) => {
        const group = document.querySelector('#airspeed');
        const trend = document.querySelector('#airspeed-trend');
        return group.getAttribute('aria-label') === `Indicated airspeed ${ias} knots` &&
          group.dataset.overspeed === String(overspeed) && group.dataset.approaching === String(approaching) &&
          (prediction === null ? !trend.hasAttribute('data-prediction-kt') :
            Math.abs(Number(trend.dataset.predictionKt) - prediction) < 1e-8);
      }, { ias, prediction, overspeed, approaching });
      const geometry = await page.evaluate(() => {
        const group = document.querySelector('#airspeed');
        const window = group.querySelector('.airspeed-window');
        const windowBox = window.getBBox();
        const center = windowBox.y + windowBox.height / 2;
        const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
        const trend = document.querySelector('#airspeed-trend');
        const trendBox = trend.getBBox();
        const digitClip = document.querySelector('#airspeed-digits-clip rect').getBBox();
        const digits = [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.getBBox());
        // Column boxes are local; apply their translations to check painted horizontal extents.
        const digitsInside = [...document.querySelectorAll('#airspeed-digits > g')].every((n, i) => {
          const x = n.transform.baseVal.consolidate().matrix.e;
          return digits[i].x + x >= digitClip.x && digits[i].x + x + digits[i].width <= digitClip.x + digitClip.width;
        });
        return { center, clip: { y: clip.y, bottom: clip.y + clip.height },
          ranges: [...document.querySelectorAll('#airspeed-ranges rect')].map(n => {
            const b = n.getBBox(); return { name: n.dataset.range, x: b.x, y: b.y, width: b.width, height: b.height };
          }),
          windowFill: getComputedStyle(window).fill,
          digitFills: [...group.querySelectorAll('.airspeed-digit')].map(n => getComputedStyle(n).fill),
          trendVisible: getComputedStyle(trend).display !== 'none',
          trendTop: trendBox.y, trendBottom: trendBox.y + trendBox.height, digitsInside };
      });
      assert.equal(geometry.windowFill, overspeed ? 'rgb(255, 67, 67)' : 'rgb(8, 13, 18)');
      assert.ok(geometry.digitFills.every(fill => fill === (approaching ? 'rgb(255, 221, 67)' : 'rgb(245, 247, 250)')));
      assert.equal(geometry.digitsInside, true, `IAS digits fit at ${width}x${height}, ${ias} kt`);
      assert.equal(geometry.trendVisible, prediction !== null);
      if (prediction !== null) {
        assert.ok(geometry.trendTop >= geometry.clip.y + 1 - 1e-4);
        assert.ok(geometry.trendBottom <= geometry.clip.bottom - 1 + 1e-4);
      }
      for (const [index, [low, high]] of [[0, 40], [40, 85], [50, 110], [110, 140], [140, Infinity]].entries()) {
        const top = Math.max(geometry.clip.y, Math.min(geometry.clip.bottom, geometry.center + (ias - high) * 4));
        const bottom = Math.max(geometry.clip.y, Math.min(geometry.clip.bottom, geometry.center + (ias - low) * 4));
        const actual = geometry.ranges[index];
        assert.ok(Math.abs(actual.y - top) < 1e-4 && Math.abs(actual.height - (bottom - top)) < 1e-4);
      }
      assert.equal(geometry.ranges[1].x + geometry.ranges[1].width, geometry.ranges[2].x,
        'White and green occupy adjacent strips instead of overwriting each other');
    }
    for (const speed of [55, 65, 75, 90, 0, 150]) {
      setFixture({ ias_kt: speed, ias_acceleration_ktps: 0 });
      await page.waitForFunction(speed => document.querySelector('#airspeed').getAttribute('aria-label') ===
        `Indicated airspeed ${speed} knots`, speed);
      const result = await page.evaluate(() => {
        const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
        const win = document.querySelector('.airspeed-window').getBBox();
        return { top: clip.y, bottom: clip.y + clip.height, center: win.y + win.height / 2,
          markers: [...document.querySelectorAll('#airspeed-references text')].map(n => ({
            text: n.textContent, y: Number(n.getAttribute('y')), visible: getComputedStyle(n).display !== 'none',
          })) };
      });
      for (const [i, reference] of [55, 65, 75, 90].entries()) {
        const expected = result.center + (speed - reference) * 4;
        assert.ok(Math.abs(result.markers[i].y - expected) < 1e-4);
        assert.equal(result.markers[i].visible, expected >= result.top + 8 && expected <= result.bottom - 8);
      }
    }
    // Exact clipping edges and just inside/outside them, checked through actual SVG.
    const bounds = await page.evaluate(() => {
      const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
      const win = document.querySelector('.airspeed-window').getBBox();
      return { top: clip.y + 8, bottom: clip.y + clip.height - 8, center: win.y + win.height / 2 };
    });
    for (const edge of [bounds.top, bounds.bottom]) {
      for (const delta of [-0.01, 0, 0.01]) {
        const speed = 55 + (edge + delta - bounds.center) / 4;
        setFixture({ ias_kt: speed });
        await page.waitForFunction(speed => document.querySelector('#airspeed').getAttribute('aria-label') ===
          `Indicated airspeed ${speed} knots`, speed);
        const visible = await page.locator('#airspeed-references text[data-reference="R"]').evaluate(n => getComputedStyle(n).display !== 'none');
        assert.equal(visible, edge + delta >= bounds.top && edge + delta <= bounds.bottom);
      }
    }
    setFixture({ ias_kt: null, gs_kt: 137 });
    await page.locator('#airspeed-references-bottom[data-visible="true"]').waitFor();
    assert.equal(await page.locator('#airspeed-warning').textContent(), 'X');
    assert.deepEqual(await page.locator('#airspeed-references-bottom .airspeed-reference-value').allTextContents(), ['R 55', 'X 65', 'Y 75', 'G 90']);
    assert.equal(await page.locator('#ground-speed').getAttribute('data-state'), 'available');
    const bottomFits = await page.evaluate(() => {
      const group = document.querySelector('#airspeed');
      const left = group.querySelector('.instrument-background').getBoundingClientRect();
      const gs = document.querySelector('#ground-speed').getBoundingClientRect();
      const x = document.querySelector('#airspeed-warning').getBoundingClientRect();
      const rows = [...document.querySelectorAll('#airspeed-references-bottom text')].map(n => n.getBoundingClientRect());
      return { fits: rows.every(r => r.left >= left.left && r.right <= left.right && r.top > x.bottom && r.bottom < gs.top) &&
        rows.slice(1).every((r, i) => r.top > rows[i].bottom),
        rows: rows.map(r => ({ left: r.left, right: r.right, top: r.top, bottom: r.bottom })),
        left: { left: left.left, right: left.right }, gsTop: gs.top, warningBottom: x.bottom };
    });
    await page.screenshot({ path: `.local/pfd-browser/speed-references-unavailable-${width}x${height}.png` });
    assert.equal(bottomFits.fits, true, `Fallback references fit above GS and below the local X: ${JSON.stringify(bottomFits)}`);
  }
  // Valid configuration magnitudes must not enlarge the allocated instrument.
  await page.setViewportSize({ width: 320, height: 240 });
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  for (const [reference, vne, expected] of [[55.123456789, 140, 'R 55.1'],
    [1234.5678, 2000, 'R OVR'], [1e-300, 140, 'R <0.1']]) {
    setFixture({ ias_kt: null, airspeed_profile: { ...profile, vne_kt: vne,
      references: { ...profile.references, vr_kt: reference } } });
    await page.waitForFunction(expected => document.querySelector('#airspeed-references-bottom [data-reference="R"]').textContent === expected, expected);
    assert.ok((await page.locator('#airspeed-references-bottom').getAttribute('aria-label')).includes(String(reference)));
    assert.equal(await page.evaluate(() => {
      const allocation = document.querySelector('#airspeed .instrument-background').getBoundingClientRect();
      return [...document.querySelectorAll('#airspeed-references-bottom text')].every(n => {
        const b = n.getBoundingClientRect();
        return b.left - 0.5 >= allocation.left && b.right + 0.5 <= allocation.right;
      });
    }), true, `Fallback text and stroke fit for ${reference}`);
  }
  const clusteredSpeeds = await page.evaluate(() => {
    const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
    const win = document.querySelector('.airspeed-window').getBBox();
    const center = win.y + win.height / 2;
    return [60 + (clip.y + 10 - center) / 4, 60, 60 + (clip.y + clip.height - 10 - center) / 4];
  });
  for (const values of [[60, 60, 60, 60], [60, 60.1, 60.2, 60.3]]) {
    for (const ias of clusteredSpeeds) {
      setFixture({ ias_kt: ias, ias_acceleration_ktps: 0, airspeed_profile: { ...profile,
        references: Object.fromEntries(['vr_kt', 'vx_kt', 'vy_kt', 'vg_kt'].map((key, i) => [key, values[i]])) } });
      await page.waitForFunction(({ ias, value }) => document.querySelector('#airspeed').getAttribute('aria-label') === `Indicated airspeed ${ias} knots` &&
        document.querySelector('#airspeed-references [data-reference="Y"][aria-label]').getAttribute('aria-label') === `VY ${value} knots`, { ias, value: values[2] });
      const geometry = await page.evaluate(() => {
        const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
        const win = document.querySelector('.airspeed-window').getBBox();
        const markers = [...document.querySelectorAll('#airspeed-references text')].filter(n => getComputedStyle(n).display !== 'none');
        return { top: clip.y, bottom: clip.y + clip.height, center: win.y + win.height / 2,
          markers: markers.map(n => { const b = n.getBBox(); return { y: b.y, bottom: b.y + b.height, label: Number(n.getAttribute('y')), name: n.dataset.reference }; }),
          leaders: [...document.querySelectorAll('.airspeed-reference-leader')].map(n => ({ name: n.dataset.reference, d: n.getAttribute('d') })) };
      });
      assert.equal(geometry.markers.length, 4);
      const sorted = geometry.markers.toSorted((a, b) => a.y - b.y);
      assert.ok(sorted.every(m => m.y - 0.5 >= geometry.top && m.bottom + 0.5 <= geometry.bottom));
      assert.ok(sorted.slice(1).every((m, i) => m.y - 0.5 > sorted[i].bottom + 0.5), 'Nearby labels and strokes do not overlap');
      for (const [i, name] of ['R', 'X', 'Y', 'G'].entries()) {
        const marker = geometry.markers.find(m => m.name === name);
        const anchor = geometry.center + (ias - values[i]) * 4;
        const path = geometry.leaders.find(n => n.name === name).d;
        if (Math.abs(marker.label - anchor) > 1e-4)
          assert.ok(Math.abs(Number(path.match(/^M [\d.]+ (-?[\d.]+)/)[1]) - anchor) < 1e-4, 'Leader retains exact configured speed');
      }
    }
  }
  for (const [fields, prediction, ranges, overspeed] of [
    [{ ias_kt: 110, ias_acceleration_ktps: null }, null, true, false],
    [{ ias_kt: 150, ias_acceleration_ktps: '5' }, null, true, true],
    [{ ias_kt: 110, ias_acceleration_ktps: 5, airspeed_profile: null }, 140, false, false],
    [{ ias_kt: 150, ias_acceleration_ktps: 5, airspeed_profile: { ...profile, vne_kt: 0 } }, 180, false, false],
    [{ ias_kt: 999.01, ias_acceleration_ktps: 5, airspeed_profile: profile }, null, false, true],
  ]) {
    setFixture(fields);
    await page.waitForFunction(({ prediction, ranges, overspeed }) => {
      const trend = document.querySelector('#airspeed-trend');
      return (prediction === null ? !trend.hasAttribute('data-prediction-kt') : Number(trend.dataset.predictionKt) === prediction) &&
        (getComputedStyle(document.querySelector('#airspeed-ranges')).display !== 'none') === ranges &&
        document.querySelector('#airspeed').dataset.overspeed === String(overspeed);
    }, { prediction, ranges, overspeed });
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
  }
  setFixture({ ias_kt: 110, ias_acceleration_ktps: 5, airspeed_profile: profile });
  await page.locator('#airspeed[data-approaching="true"]').waitFor();
  await page.screenshot({ path: '.local/pfd-browser/speed-approaching-vne.png' });
  setFixture({ ias_kt: 140, ias_acceleration_ktps: 0 });
  await page.locator('#airspeed[data-overspeed="true"]').waitFor();
  await page.screenshot({ path: '.local/pfd-browser/speed-at-vne.png' });
  setFixture({ ias_kt: 123, gs_kt: 137, ias_acceleration_ktps: 5, airspeed_profile: profile });
}

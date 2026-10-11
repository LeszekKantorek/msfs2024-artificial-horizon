import assert from 'node:assert/strict';

// Exercise the actual instrument, transport and painted bounds in both page compositions.
export async function checkAltitude(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ altitude_ft: 1000, baro_inhg: 29.92, hold: false, attitude_only: false, state: 'live', pitch_deg: 0, roll_deg: 0 });
    await page.goto(url + path);
    await page.locator('#altitude[data-state="available"]').waitFor();
    await page.evaluate(() => {
      window.altitudeNodes = [...document.querySelectorAll('#altitude *, #barometric-setting *, #altitude-warning, #altitude-tape-clip *, #altitude-digits-clip *')];
    });
    for (const [width, height] of [[326, 246], [320, 480], [568, 240], [844, 320]]) {
      await page.setViewportSize({ width, height });
      for (const value of [-9999, -1000, -90, -20, -0.1, 0, 0.1, 20, 90, 100, 990, 1000, 9990, 10000, 99999]) {
        setFixture({ altitude_ft: value });
        await page.waitForFunction(value => document.querySelector('#altitude').getAttribute('aria-label') === `Barometric altitude ${value} feet`, value);
        const actual = await page.evaluate(() => {
          const group = document.querySelector('#altitude');
          const clip = document.querySelector('#altitude-tape-clip rect').getBBox();
          const center = clip.height / 2;
          const columns = [...document.querySelectorAll('#altitude-digits > g')];
          const digitsClip = document.querySelector('#altitude-digits-clip rect').getBBox();
          const sign = document.querySelector('#altitude-sign');
          const digitBoxes = [sign, ...columns.map(n => n.firstElementChild)]
            .filter(n => n.textContent).map(n => {
              const box = n.getBBox();
              const matrix = n.parentElement === sign.parentElement ? new DOMMatrix() : n.parentElement.transform.baseVal.consolidate().matrix;
              return { left: box.x + matrix.e, right: box.x + box.width + matrix.e };
            }).sort((a, b) => a.left - b.left);
          const baro = document.querySelector('#barometric-setting .instrument-background').getBoundingClientRect();
          const baroTexts = [...document.querySelectorAll('#barometric-setting text')].map(n => n.getBoundingClientRect());
          const panel = document.querySelector('svg').getBoundingClientRect();
          const background = group.querySelector('.instrument-background').getBoundingClientRect();
          const painted = [...document.querySelectorAll('#altitude-scale text')].filter(n => n.textContent);
          const labelsClearOfTicks = painted.every(n => n.getBBox().x > 13);
          const initial = window.altitudeNodes;
          const nodes = [...document.querySelectorAll('#altitude *, #barometric-setting *, #altitude-warning, #altitude-tape-clip *, #altitude-digits-clip *')];
          return { sign: sign.textContent, current: columns.map(n => n.firstElementChild.textContent),
            offsets: columns.map(n => n.transform.baseVal.consolidate().matrix.f - center),
            height: clip.height, ticks: [...document.querySelectorAll('.altitude-tick')].map(n => ({
              value: Number(n.dataset.altitudeFt), y: Number(n.getAttribute('d').split(' ')[2]) })),
            digitsInside: digitBoxes.every(b => b.left >= digitsClip.x && b.right <= digitsClip.x + digitsClip.width),
            digitOverlap: digitBoxes.slice(1).some((b, i) => b.left < digitBoxes[i].right),
            baroContained: baroTexts.every(b => b.left >= baro.left && b.right <= baro.right && b.top >= baro.top && b.bottom <= baro.bottom),
            baroGap: baroTexts[1].top - baroTexts[0].bottom,
            baroTouchesBackground: Math.abs(baro.top - background.bottom) < 0.01,
            panelContained: baro.right <= panel.right && baro.bottom <= panel.bottom && background.left >= panel.left,
            labelsClearOfTicks, unchanged: nodes.length === initial.length && nodes.every((n, i) => n === initial[i]),
            opacity: getComputedStyle(group.querySelector('.instrument-background')).fillOpacity,
            digitsOpacity: getComputedStyle(document.querySelector('.altitude-digit')).fillOpacity };
        });
        assert.equal(actual.sign, value < 0 ? '−' : '');
        assert.ok(actual.digitsInside, `Digits clipped at ${value}, ${width}x${height}`);
        assert.equal(actual.digitOverlap, false, `Digits overlap at ${value}, ${width}x${height}`);
        assert.ok(actual.baroContained && actual.panelContained && actual.baroGap >= 0,
          JSON.stringify({ value, width, height, baroContained: actual.baroContained, panelContained: actual.panelContained, baroGap: actual.baroGap }));
        assert.ok(actual.baroTouchesBackground && actual.labelsClearOfTicks && actual.unchanged);
        assert.equal(actual.opacity, '0.65'); assert.equal(actual.digitsOpacity, '1');
        for (const tick of actual.ticks) {
          const expected = actual.height / 2 + (value - tick.value) * actual.height / 400;
          assert.ok(Math.abs(tick.y - expected) < 1e-8);
        }
        if (value === 90) {
          assert.deepEqual(actual.current, ['', '', '', '80']);
          assert.deepEqual(actual.offsets, [0, 0, 10, 10]);
        }
        if (value === 9990) {
          assert.deepEqual(actual.current, ['', '9', '9', '80']);
          assert.deepEqual(actual.offsets, [10, 10, 10, 10]);
        }
      }
    }
    // Local loss and overflow preserve the other indication and the live panel.
    for (const [altitude, baro, altitudeState, baroState, text] of [
      [null, 29.92, 'unavailable', 'available', '29.92'],
      [-120, null, 'available', 'unavailable', 'X'],
      ['bad', 30.12, 'unavailable', 'available', '30.12'],
      [20, 0, 'available', 'unavailable', 'X'],
      [-10000, 29.92, 'overflow', 'available', '29.92'],
      [100000, 100, 'overflow', 'overflow', 'OVR'],
      [0, 29.72, 'available', 'available', '29.72'],
    ]) {
      setFixture({ altitude_ft: altitude, baro_inhg: baro });
      await page.waitForFunction(({ altitudeState, baroState, text }) =>
        document.querySelector('#altitude').dataset.state === altitudeState &&
        document.querySelector('#barometric-setting').dataset.state === baroState &&
        document.querySelector('#barometric-value').textContent === text, { altitudeState, baroState, text });
      assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
      assert.equal(await page.locator('.altitude-tape').evaluate(n => getComputedStyle(n).visibility), altitudeState === 'available' ? 'visible' : 'hidden');
      assert.equal(await page.locator('#altitude-digits').evaluate(n => getComputedStyle(n).visibility), altitudeState === 'available' ? 'visible' : 'hidden');
      assert.equal(await page.locator('#altitude-warning').textContent(), altitudeState === 'overflow' ? 'OVR' : 'X');
    }
    for (const [width, height] of [[326, 246], [320, 480], [568, 240]]) {
      await page.setViewportSize({ width, height });
      for (const altitude of [-9980, 99980]) {
        for (const pitch of [40, -40]) {
          setFixture({ pitch_deg: pitch, altitude_ft: altitude, baro_inhg: 29.92 });
          await page.waitForFunction(({ altitude, pitch }) => {
            const world = document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f;
            return document.querySelector('#altitude').getAttribute('aria-label') === `Barometric altitude ${altitude} feet` &&
              Math.sign(world) === Math.sign(pitch);
          }, { altitude, pitch });
          await page.screenshot({ path: `.local/pfd-browser/altitude-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${altitude}-${pitch}.png` });
        }
      }
    }
    setFixture({ attitude_only: true });
    await page.locator('#altitude[data-state="unavailable"]').waitFor();
    assert.equal(await page.locator('#barometric-setting').getAttribute('data-state'), 'unavailable');
    setFixture({ attitude_only: false, altitude_ft: 1000, baro_inhg: 29.92, pitch_deg: 0 });
    await page.locator('#altitude[data-state="available"]').waitFor();
    setFixture({ hold: true });
    await page.locator('.instrument[data-available="false"]').waitFor();
    assert.equal(await page.locator('#altitude').evaluate(n => getComputedStyle(n).visibility), 'hidden');
    await page.setViewportSize({ width: 320, height: 480 });
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
    setFixture({ hold: false, altitude_ft: -20, baro_inhg: 30.12 });
    await page.waitForFunction(() => document.querySelector('.instrument').dataset.available === 'true' &&
      document.querySelector('#altitude').getAttribute('aria-label') === 'Barometric altitude -20 feet');
  }
  // Return to the production page for the existing whole-panel checks.
  setFixture({ altitude_ft: 10000, baro_inhg: 29.92, pitch_deg: 0, roll_deg: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

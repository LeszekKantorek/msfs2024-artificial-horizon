import assert from 'node:assert/strict';

export async function checkVerticalSpeed(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', attitude_only: false, altitude_ft: 10000, selected_altitude_ft: 12000,
      baro_inhg: 29.92, vertical_speed_fpm: 0, hold: false, pitch_deg: 0 });
    await page.goto(url + path);
    await page.locator('#vertical-speed[data-state="available"]').waitFor();
    await page.evaluate(() => { window.vsiNodes = [...document.querySelectorAll('#vertical-speed *, #vertical-speed-clip *')]; });
    for (const [width, height] of [[320, 240], [320, 480], [568, 240], [326, 246]]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      const geometry = await page.evaluate(() => ({ height: document.querySelector('#vertical-speed-clip rect').getBBox().height,
        scale: document.querySelector('#vertical-speed-scale').innerHTML }));
      for (const [fpm, text, state, offScale] of [[0, '0', 'available', false], [-0.1, '0', 'available', false],
        [1500, '+1500', 'available', false], [-1500, '-1500', 'available', false],
        [2000, '+2000', 'available', false], [-2000, '-2000', 'available', false],
        [2000.01, '+2000', 'available', true], [-2000.01, '-2000', 'available', true],
        [4881.36, '+4881', 'available', true], [9999, '+9999', 'available', true], [-9999, '-9999', 'available', true],
        [9999.01, '+OVR', 'overflow', true], [-9999.01, '-OVR', 'overflow', true],
        [Number.MAX_VALUE, '+OVR', 'overflow', true], [-Number.MAX_VALUE, '-OVR', 'overflow', true]]) {
        setFixture({ vertical_speed_fpm: fpm });
        await page.waitForFunction(({ text, offScale, fpm }) => document.querySelector('#vertical-speed-value').textContent === text &&
          document.querySelector('#vertical-speed').dataset.offScale === String(offScale) &&
          document.querySelector('#vertical-speed').getAttribute('aria-label') ===
            `Vertical speed ${fpm} feet per minute${offScale ? ', off scale' : ''}`, { text, offScale, fpm });
        const painted = await page.evaluate(() => {
          const group = document.querySelector('#vertical-speed');
          const pointer = document.querySelector('#vertical-speed-pointer');
          const value = document.querySelector('#vertical-speed-value');
          const box = document.querySelector('.vertical-speed-window').getBoundingClientRect();
          const valueBox = value.getBoundingClientRect();
          const unitBox = document.querySelector('.vertical-speed-unit').getBoundingClientRect();
          const unitOrigin = group.getScreenCTM();
          const view = document.querySelector('svg').getBoundingClientRect();
          const arrow = document.querySelector('.vertical-speed-arrow');
          const bounds = arrow.getBBox();
          const clip = document.querySelector('#vertical-speed-clip rect').getBBox();
          const matrix = pointer.transform.baseVal.consolidate().matrix;
          const screen = new DOMPoint(6, 0).matrixTransform(arrow.getScreenCTM());
          const marker = document.querySelector('#vertical-speed-off-scale');
          const markerBox = marker.getBBox();
          const markerTip = new DOMPoint(5, Number(group.getAttribute('aria-label').split(' ')[2]) > 0 ? -6 : 6)
            .matrixTransform(marker.getScreenCTM());
          const scaleLabelsContained = [...document.querySelectorAll('.vertical-speed-label')].every(label => {
            const b = label.getBBox();
            return b.x >= 0 && b.x + b.width <= clip.width && b.y >= 0 && b.y + b.height <= clip.height;
          });
          return { state: group.dataset.state, aria: group.getAttribute('aria-label'), y: matrix.f,
            textContained: valueBox.left > box.left && valueBox.right < box.right && valueBox.top > box.top && valueBox.bottom < box.bottom,
            unitContained: unitBox.left >= unitOrigin.e && unitBox.right <= view.right && unitBox.top >= view.top && unitBox.bottom <= unitOrigin.f,
            arrowContained: bounds.y + matrix.f - 0.5 >= 0 && bounds.y + bounds.height + matrix.f + 0.5 <= clip.height,
            markerContained: markerBox.y + matrix.f - 0.5 >= 0 && markerBox.y + markerBox.height + matrix.f + 0.5 <= clip.height,
            paintVisible: document.elementFromPoint(screen.x, screen.y) === arrow,
            markerPaintVisible: document.elementFromPoint(markerTip.x, markerTip.y) === marker, scaleLabelsContained,
            markerDisplay: getComputedStyle(marker).display, color: getComputedStyle(value).fill,
            stable: window.vsiNodes.every(n => n.isConnected), scale: document.querySelector('#vertical-speed-scale').innerHTML };
        });
        assert.equal(painted.state, state);
        assert.equal(painted.aria, `Vertical speed ${fpm} feet per minute${offScale ? ', off scale' : ''}`);
        assert.ok(painted.textContained && painted.unitContained && painted.arrowContained && painted.paintVisible && painted.stable && painted.scaleLabelsContained,
          JSON.stringify({ width, height, fpm, painted }));
        if (offScale) assert.ok(painted.markerContained && painted.markerPaintVisible);
        assert.equal(painted.markerDisplay, offScale ? 'inline' : 'none');
        assert.equal(painted.color, 'rgb(245, 247, 250)');
        assert.equal(painted.scale, geometry.scale);
        const expected = geometry.height / 2 - Math.max(-2000, Math.min(2000, fpm)) / 2000 * (geometry.height / 2 - 8);
        assert.ok(Math.abs(painted.y - expected) < 1e-4);
      }
      await page.screenshot({ path: `.local/pfd-browser/vsi-${path === '/' ? 'production' : 'layout'}-${width}x${height}-overflow.png` });
      setFixture({ vertical_speed_fpm: 1500, altitude_ft: null, baro_inhg: null });
      await page.locator('#altitude[data-state="unavailable"]').waitFor();
      await page.waitForFunction(() => document.querySelector('#vertical-speed-value').textContent === '+1500');
      assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'available');
      for (const invalid of [null, 'bad', false]) {
        setFixture({ vertical_speed_fpm: invalid, altitude_ft: 10000, baro_inhg: 29.92 });
        await page.locator('#vertical-speed[data-state="unavailable"]').waitFor();
        assert.equal(await page.locator('#vertical-speed-value').textContent(), 'X');
        assert.equal(await page.locator('#vertical-speed-pointer').evaluate(n => getComputedStyle(n).display), 'none');
        assert.equal(await page.locator('#vertical-speed-value').evaluate(n => getComputedStyle(n).fill), 'rgb(255, 150, 144)');
        assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
      }
      setFixture({ vertical_speed_fpm: -1500 });
      await page.locator('#vertical-speed[data-state="available"]').waitFor();
      await page.screenshot({ path: `.local/pfd-browser/vsi-${path === '/' ? 'production' : 'layout'}-${width}x${height}-descent.png` });
      setFixture({ state: 'paused' });
      await page.locator('.instrument[data-available="false"]').waitFor();
      assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'unavailable');
      setFixture({ state: 'live', vertical_speed_fpm: 0 });
      await page.locator('.instrument[data-available="true"]').waitFor();
      setFixture({ hold: true });
      await page.locator('.instrument[data-available="false"]').waitFor();
      assert.equal(await page.locator('#vertical-speed-pointer').evaluate(n => getComputedStyle(n).display), 'none');
      setFixture({ hold: false, vertical_speed_fpm: 0 });
      await page.locator('.instrument[data-available="true"]').waitFor();
    }
  }
  setFixture({ vertical_speed_fpm: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

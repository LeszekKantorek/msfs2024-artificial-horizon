import assert from 'node:assert/strict';

export async function checkAltitudeTrend(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', attitude_only: false, altitude_ft: 10060, selected_altitude_ft: 10000,
      vertical_speed_fpm: 1500, baro_inhg: 29.92, hold: false });
    await page.goto(url + path);
    await page.locator('#vertical-speed[data-state="available"]').waitFor();
    await page.evaluate(() => { window.trendNode = document.querySelector('#altitude-trend'); });
    for (const [width, height] of [[320, 240], [320, 480], [568, 240], [326, 246]]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      for (const fpm of [1500, -1500, 10000, -10000]) {
        setFixture({ vertical_speed_fpm: fpm });
        await page.waitForFunction(fpm => document.querySelector('#vertical-speed').getAttribute('aria-label') === `Vertical speed ${fpm} feet per minute`, fpm);
        const paint = await page.evaluate(() => {
          const trend = document.querySelector('#altitude-trend');
          const start = trend.getPointAtLength(0), end = trend.getPointAtLength(trend.getTotalLength());
          const bounds = trend.getBoundingClientRect();
          const clip = document.querySelector('#altitude-trend-clip rect').getBBox();
          const clipStart = new DOMPoint(clip.x, clip.y).matrixTransform(trend.getScreenCTM());
          const clipEnd = new DOMPoint(clip.x + clip.width, clip.y + clip.height).matrixTransform(trend.getScreenCTM());
          const altitude = document.querySelector('.altitude-window').getBoundingClientRect();
          const vsi = document.querySelector('.vertical-speed-window').getBoundingClientRect();
          const bug = document.querySelector('#selected-altitude-bug').getBoundingClientRect();
          const style = getComputedStyle(trend), ias = getComputedStyle(document.querySelector('#airspeed-trend'));
          const point = new DOMPoint(start.x, (start.y + end.y) / 2).matrixTransform(trend.getScreenCTM());
          const height = document.querySelector('#altitude-tape-clip rect').getBBox().height;
          return { start: start.y, end: end.y, height, prediction: Number(trend.dataset.predictionFt),
            width: clip.width, painted: document.elementFromPoint(point.x, point.y) === trend,
            contained: bounds.left - 1 >= clipStart.x && bounds.right + 1 <= clipEnd.x && bounds.top >= clipStart.y && bounds.bottom <= clipEnd.y,
            separated: bounds.left - 1 > Math.max(altitude.right, vsi.right, bug.right) + 0.5,
            sameStyle: style.stroke === ias.stroke && style.strokeWidth === ias.strokeWidth,
            stable: trend === window.trendNode };
        });
        assert.equal(paint.prediction, 10060 + fpm / 10);
        assert.equal(paint.start, paint.height / 2);
        assert.ok(Math.abs(paint.end - Math.max(1, Math.min(paint.height - 1, paint.start - fpm / 10 * 0.4))) < 1e-4);
        assert.ok(paint.painted && paint.contained && paint.separated && paint.sameStyle && paint.stable && paint.width === 4,
          JSON.stringify({ width, height, fpm, paint }));
        if (Math.abs(fpm) === 1500) await page.screenshot({ path: `.local/pfd-browser/altitude-trend-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${fpm > 0 ? 'climb' : 'descent'}.png` });
      }
      // The cyan symbol may cross the window's raw bounds, but cannot cover its fill or border.
      for (const target of [10010, 10060, 10110]) {
        setFixture({ selected_altitude_ft: target });
        await page.waitForFunction(target => document.querySelector('#selected-altitude-value').textContent === String(target), target);
        const covered = await page.evaluate(() => {
          const window = document.querySelector('.altitude-window');
          const center = document.querySelector('#altitude-tape-clip rect').getBBox().height / 2;
          return [[4, 0], [8, -10], [8, 10]].every(([x, y]) => {
            const point = new DOMPoint(x, center + y).matrixTransform(window.getScreenCTM());
            return document.elementFromPoint(point.x, point.y) === window;
          });
        });
        assert.ok(covered, `Readout paint order at target ${target}, ${width}x${height}`);
      }
      for (const [altitude, fpm] of [[null, 1500], [100000, 1500], [10060, null], [10060, 0]]) {
        setFixture({ altitude_ft: altitude, vertical_speed_fpm: fpm });
        await page.waitForFunction(({ altitude, fpm }) => {
          const expected = altitude === null ? 'unavailable' : altitude > 99999 ? 'overflow' : 'available';
          return document.querySelector('#altitude').dataset.state === expected &&
            document.querySelector('#vertical-speed').getAttribute('aria-label') === (fpm === null ? 'Vertical speed unavailable' : `Vertical speed ${fpm} feet per minute`);
        }, { altitude, fpm });
        assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'none');
      }
      setFixture({ altitude_ft: 10060, vertical_speed_fpm: 1500, baro_inhg: null, selected_altitude_ft: 10000 });
      await page.locator('#barometric-setting[data-state="unavailable"]').waitFor();
      await page.waitForFunction(() => document.querySelector('#altitude-trend').dataset.predictionFt === '10210');
      assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'inline');
    }
    setFixture({ hold: true });
    await page.locator('.instrument[data-available="false"]').waitFor();
    assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'none');
    assert.equal(await page.locator('#altitude-trend').getAttribute('d'), null);
    setFixture({ hold: false, state: 'paused' });
    await page.waitForFunction(() => document.querySelector('#telemetry-status').textContent.includes('paused'));
    assert.equal(await page.locator('#altitude-trend').getAttribute('d'), null);
    setFixture({ state: 'live', baro_inhg: 29.92 });
    await page.locator('.instrument[data-available="true"]').waitFor();
    await page.waitForFunction(() => document.querySelector('#altitude-trend').dataset.predictionFt === '10210');
    assert.equal(await page.locator('#altitude-trend').evaluate(n => n === window.trendNode), true);
  }
  setFixture({ vertical_speed_fpm: 0, selected_altitude_ft: 12000, altitude_ft: 10000 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

import assert from 'node:assert/strict';

export async function checkAltitudeTrend(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', attitude_only: false, altitude_ft: 10060, selected_altitude_ft: 10000,
      vertical_speed_fpm: 1500, baro_text: '29.92 inHg', hold: false });
    await page.goto(url + path);
    await page.locator('#vertical-speed[data-state="available"]').waitFor();
    await page.evaluate(() => { window.trendNode = document.querySelector('#altitude-trend'); });
    for (const [width, height] of [[320, 240], [326, 246], [320, 480], [390, 664], [568, 240], [667, 280], [844, 320]]) {
      await page.setViewportSize({ width, height });
      await page.waitForFunction(() => {
        const svg = document.querySelector('svg'), box = svg.getBoundingClientRect();
        const [,, width, height] = svg.getAttribute('viewBox').split(' ').map(Number);
        return Math.abs(width - box.width) < 0.01 && Math.abs(height - box.height) < 0.01;
      });
      for (const fpm of [1500, -1500, 10000, -10000]) {
        setFixture({ altitude_ft: 10060, selected_altitude_ft: null, vertical_speed_fpm: fpm, baro_text: '29.92 inHg' });
        await page.waitForFunction(fpm => document.querySelector('#vertical-speed').getAttribute('aria-label') === `Vertical speed ${fpm} feet per minute` &&
          document.querySelector('#selected-altitude').dataset.state === 'unavailable' &&
          document.querySelector('#altitude').getAttribute('aria-label') === 'Barometric altitude 10060 feet', fpm);
        const paint = await page.evaluate(() => {
          const trend = document.querySelector('#altitude-trend');
          const start = trend.getPointAtLength(0), end = trend.getPointAtLength(trend.getTotalLength());
          const bounds = trend.getBoundingClientRect();
          const clip = document.querySelector('#altitude-trend-clip rect').getBBox();
          const clipStart = new DOMPoint(clip.x, clip.y).matrixTransform(trend.getScreenCTM());
          const clipEnd = new DOMPoint(clip.x + clip.width, clip.y + clip.height).matrixTransform(trend.getScreenCTM());
          const style = getComputedStyle(trend), ias = getComputedStyle(document.querySelector('#airspeed-trend'));
          const tick = [...document.querySelectorAll('.altitude-tick')].find(node => {
            if (getComputedStyle(node).display === 'none') return false;
            const y = node.getPointAtLength(0).y;
            return y > Math.min(start.y, end.y) + 2 && y < Math.max(start.y, end.y) - 2 && Math.abs(y - start.y) > 16;
          });
          const local = new DOMPoint(start.x, tick.getPointAtLength(0).y);
          const point = local.matrixTransform(trend.getScreenCTM());
          const coversTick = tick.isPointInStroke(local) && trend.isPointInStroke(local) &&
            document.elementFromPoint(point.x, point.y) === trend;
          // The same painted point must detect an incorrectly lowered trend.
          const next = trend.nextSibling, parent = trend.parentNode;
          let rejectsUnderTicks;
          try {
            parent.insertBefore(trend, document.querySelector('.altitude-tape'));
            rejectsUnderTicks = document.elementFromPoint(point.x, point.y) === tick;
          } finally { parent.insertBefore(trend, next); }
          const height = document.querySelector('#altitude-tape-clip rect').getBBox().height;
          return { x: start.x, start: start.y, end: end.y, height, prediction: Number(trend.dataset.predictionFt),
            width: clip.width, coversTick, rejectsUnderTicks,
            contained: bounds.left - 1 >= clipStart.x && bounds.right + 1 <= clipEnd.x && bounds.top >= clipStart.y && bounds.bottom <= clipEnd.y,
            sameStyle: style.stroke === ias.stroke && style.strokeWidth === ias.strokeWidth,
            stable: trend === window.trendNode };
        });
        assert.equal(paint.prediction, 10060 + fpm / 10);
        assert.equal(paint.start, paint.height / 2);
        assert.ok(Math.abs(paint.end - Math.max(1, Math.min(paint.height - 1, paint.start - fpm / 10 * 0.4))) < 1e-4);
        assert.equal(paint.x, 2);
        assert.ok(paint.coversTick && paint.rejectsUnderTicks && paint.contained && paint.sameStyle && paint.stable && paint.width === 4,
          JSON.stringify({ width, height, fpm, paint }));
        if (Math.abs(fpm) === 1500) await page.screenshot({ path: `.local/pfd-browser/altitude-trend-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${fpm > 0 ? 'climb' : 'descent'}.png` });
      }
      for (const [altitude, target, fpm] of [[10060, 10060, 1500], [10060, 10060, -1500],
        [-120, -120, 1500], [-120, -120, -1500], [10060, 12000, 10000], [10060, 8000, -10000]]) {
        setFixture({ altitude_ft: altitude, selected_altitude_ft: target, vertical_speed_fpm: fpm });
        await page.waitForFunction(({ altitude, target, fpm }) =>
          document.querySelector('#altitude').getAttribute('aria-label') === `Barometric altitude ${altitude} feet` &&
          document.querySelector('#selected-altitude-value').textContent === String(target) &&
          document.querySelector('#vertical-speed').getAttribute('aria-label') === `Vertical speed ${fpm} feet per minute`, { altitude, target, fpm });
        const paint = await page.evaluate(fpm => {
          const trend = document.querySelector('#altitude-trend'), bug = document.querySelector('#selected-altitude-bug');
          const bugY = bug.transform.baseVal.consolidate().matrix.f;
          const local = new DOMPoint(2, bugY + (fpm > 0 ? -4 : 4));
          const point = local.matrixTransform(trend.getScreenCTM());
          const bugPoint = new DOMPoint(2, local.y - bugY);
          const coversTrend = trend.isPointInStroke(local) && bug.isPointInFill(bugPoint) &&
            document.elementFromPoint(point.x, point.y) === bug;
          const next = trend.nextSibling, parent = trend.parentNode;
          let rejectsOverBug;
          try {
            parent.insertBefore(trend, document.querySelector('.altitude-window'));
            rejectsOverBug = document.elementFromPoint(point.x, point.y) === trend;
          } finally { parent.insertBefore(trend, next); }
          const height = document.querySelector('#altitude-tape-clip rect').getBBox().height;
          return { coversTrend, rejectsOverBug, bugY, height, color: getComputedStyle(bug).fill,
            stable: trend === window.trendNode };
        }, fpm);
        assert.ok(paint.coversTrend && paint.rejectsOverBug && paint.stable, JSON.stringify({ width, height, altitude, target, fpm, paint }));
        assert.equal(paint.color, 'rgb(134, 222, 245)');
        if (target === altitude) assert.equal(paint.bugY, paint.height / 2);
        else assert.equal(paint.bugY, target > altitude ? 14.5 : paint.height - 14.5);
      }
      // The cyan symbol may cross the window's raw bounds, but cannot cover its fill or border.
      for (const target of [10010, 10060, 10110]) {
        setFixture({ altitude_ft: 10060, selected_altitude_ft: target, vertical_speed_fpm: -1500 });
        await page.waitForFunction(target => document.querySelector('#selected-altitude-value').textContent === String(target) &&
          document.querySelector('#vertical-speed').getAttribute('aria-label') === 'Vertical speed -1500 feet per minute', target);
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
      const windows = await page.evaluate(() => {
        const window = document.querySelector('.altitude-window'), trend = document.querySelector('#altitude-trend');
        const center = document.querySelector('#altitude-tape-clip rect').getBBox().height / 2;
        const local = new DOMPoint(2.8, center), point = local.matrixTransform(trend.getScreenCTM());
        const windowCoversTrend = window.isPointInStroke(local) && trend.isPointInStroke(local) &&
          document.elementFromPoint(point.x, point.y) === window;
        const next = trend.nextSibling, parent = trend.parentNode;
        let rejectsOverWindow;
        try {
          parent.append(trend);
          rejectsOverWindow = document.elementFromPoint(point.x, point.y) === trend;
        } finally { parent.insertBefore(trend, next); }
        const vsi = document.querySelector('.vertical-speed-window').getBoundingClientRect();
        const text = document.querySelector('#vertical-speed-value').getBoundingClientRect();
        const bug = document.querySelector('#selected-altitude-bug'), host = bug.parentNode, hostNext = host.nextSibling;
        const bugMatrix = bug.getScreenCTM(), borderPoint = new DOMPoint(6, 10).matrixTransform(bugMatrix);
        const windowCoversBug = document.elementFromPoint(borderPoint.x, borderPoint.y) === window;
        let rejectsBugOverWindow;
        try {
          parent.append(host);
          rejectsBugOverWindow = document.elementFromPoint(borderPoint.x, borderPoint.y) === bug;
        } finally { parent.insertBefore(host, hostNext); }
        return { windowCoversTrend, rejectsOverWindow, windowCoversBug, rejectsBugOverWindow,
          vsiClear: text.left > vsi.left && text.right < vsi.right && text.top > vsi.top && text.bottom < vsi.bottom };
      });
      assert.ok(Object.values(windows).every(Boolean), JSON.stringify({ width, height, windows }));
      for (const [altitude, fpm] of [[null, 1500], [100000, 1500], [10060, null], [10060, 0]]) {
        setFixture({ altitude_ft: altitude, vertical_speed_fpm: fpm });
        await page.waitForFunction(({ altitude, fpm }) => {
          const expected = altitude === null ? 'unavailable' : altitude > 99999 ? 'overflow' : 'available';
          return document.querySelector('#altitude').dataset.state === expected &&
            document.querySelector('#vertical-speed').getAttribute('aria-label') === (fpm === null ? 'Vertical speed unavailable' : `Vertical speed ${fpm} feet per minute`);
        }, { altitude, fpm });
        assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'none');
      }
      setFixture({ altitude_ft: 10060, vertical_speed_fpm: 1500, baro_text: null, selected_altitude_ft: null });
      await page.locator('#barometric-setting[data-state="unavailable"]').waitFor();
      await page.locator('#selected-altitude[data-state="unavailable"]').waitFor();
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
    setFixture({ state: 'live', baro_text: '29.92 inHg' });
    await page.locator('.instrument[data-available="true"]').waitFor();
    await page.waitForFunction(() => document.querySelector('#altitude-trend').dataset.predictionFt === '10210');
    assert.equal(await page.locator('#altitude-trend').evaluate(n => n === window.trendNode), true);
  }
  setFixture({ vertical_speed_fpm: 0, selected_altitude_ft: 12000, altitude_ft: 10000 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

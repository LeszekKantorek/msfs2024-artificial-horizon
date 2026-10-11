import assert from 'node:assert/strict';

export async function checkTrueAirspeed(page, setFixture, url) {
  const resizeComplete = () => page.waitForFunction(() => {
    const svg = document.querySelector('svg'), b = svg.getBoundingClientRect();
    const [, , w, h] = svg.getAttribute('viewBox').split(' ').map(Number);
    return Math.abs(w - b.width) < 0.01 && Math.abs(h - b.height) < 0.01;
  });
  async function reading(speed, state, text) {
    setFixture({ tas_kt: speed, omit_tas: false });
    await page.waitForFunction(({ state, text }) => document.querySelector('#true-airspeed').dataset.state === state &&
      document.querySelector('#true-airspeed-value').textContent === text, { state, text });
    assert.equal(await page.locator('#true-airspeed').getAttribute('aria-label'),
      `True airspeed ${state === 'available' ? `${text} knots` : state}`);
    const geometry = await page.evaluate(() => {
      const rect = selector => {
        const r = document.querySelector(selector).getBoundingClientRect();
        return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, width: r.width, height: r.height };
      };
      const bg = rect('.true-airspeed-background'), border = rect('.true-airspeed-border');
      const label = rect('.true-airspeed-label'), value = rect('#true-airspeed-value');
      const panel = rect('svg'), tape = rect('#airspeed .instrument-background');
      const style = selector => {
        const s = getComputedStyle(document.querySelector(selector));
        return [s.fontFamily, s.fontSize, s.fontWeight, s.fill, s.dominantBaseline];
      };
      const background = getComputedStyle(document.querySelector('.true-airspeed-background'));
      const stroke = getComputedStyle(document.querySelector('.true-airspeed-border'));
      return { bg, label, value, panel, tape,
        insets: [border.left - bg.left, border.top - bg.top, bg.right - border.right, bg.bottom - border.bottom],
        labelStyle: style('.true-airspeed-label'), valueStyle: style('#true-airspeed-value'),
        gsFont: getComputedStyle(document.querySelector('.ground-speed-label')).fontFamily,
        background: [background.fill, background.fillOpacity], border: [stroke.stroke, stroke.strokeWidth],
        centerY: Number(document.querySelector('#true-airspeed-value').getAttribute('y')),
      };
    });
    const { bg, label, value, panel, tape } = geometry;
    assert.ok(label.right < value.left, JSON.stringify(geometry));
    for (const r of [label, value]) assert.ok(r.left >= bg.left + 1 && r.right <= bg.right - 1 &&
      r.top >= bg.top + 1 && r.bottom <= bg.bottom - 1, JSON.stringify(geometry));
    assert.equal(bg.height, 32);
    assert.ok(Math.abs(bg.left - panel.left) < 0.01 && Math.abs(bg.top - panel.top) < 0.01);
    assert.ok(Math.abs(bg.width - tape.width) < 0.01 && Math.abs(bg.bottom - tape.top) < 0.01);
    assert.ok(geometry.insets.every(n => Math.abs(n - 0.5) < 0.01));
    assert.deepEqual(geometry.background, ['rgb(8, 13, 18)', '1']);
    assert.deepEqual(geometry.border, ['rgb(245, 247, 250)', '1px']);
    assert.deepEqual(geometry.labelStyle, [geometry.gsFont, '11px', '600', 'rgb(245, 247, 250)', 'central']);
    assert.deepEqual(geometry.valueStyle.slice(0, 3), geometry.labelStyle.slice(0, 3));
    assert.equal(geometry.valueStyle[3], state === 'available' ? 'rgb(245, 247, 250)' : 'rgb(255, 150, 144)');
    assert.equal(geometry.centerY, 16);
    assert.ok(Math.abs(label.top - value.top) < 0.1);
    assert.equal(await page.locator('.true-airspeed-label').textContent(), 'TAS');
    const collisions = await page.evaluate(() => {
      const heading = document.querySelector('#heading');
      const background = document.querySelector('.true-airspeed-background').getBoundingClientRect();
      const clip = document.querySelector('#heading-tape-clip rect').getBBox();
      const topLeft = new DOMPoint(clip.x, clip.y).matrixTransform(heading.getScreenCTM());
      const bottomRight = new DOMPoint(clip.x + clip.width, clip.y + clip.height).matrixTransform(heading.getScreenCTM());
      return [...heading.querySelectorAll('text, path')].filter(n => {
        const s = getComputedStyle(n);
        if (s.display === 'none' || s.visibility !== 'visible') return false;
        const r = n.getBoundingClientRect();
        if (!r.width && !r.height) return false;
        const stroke = s.stroke === 'none' ? 0 : parseFloat(s.strokeWidth) / 2;
        const left = Math.max(r.left - stroke, topLeft.x), right = Math.min(r.right + stroke, bottomRight.x);
        const top = Math.max(r.top - stroke, topLeft.y), bottom = Math.min(r.bottom + stroke, bottomRight.y);
        return right > left && bottom > top && left < background.right && right > background.left &&
          top < background.bottom && bottom > background.top;
      }).map(n => n.id || n.getAttribute('class'));
    });
    assert.deepEqual(collisions, []);
  }
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', hold: false, malformed: false, attitude_only: false, omit_tas: false,
      ias_kt: 150, gs_kt: 170, tas_kt: 160, heading_deg: 359, selected_heading_deg: 1, pitch_deg: 0, roll_deg: 0 });
    await page.goto(url + path);
    await page.locator('.instrument[data-available="true"]').waitFor();
    await page.evaluate(() => { window.tasNodes = [...document.querySelectorAll('#true-airspeed, #true-airspeed *')]; });
    for (const [width, height] of [[320, 480], [326, 246], [568, 240]]) {
      await page.setViewportSize({ width, height });
      for (const padding of ['3px', '3.25px', '15px 24px 12px 12px']) {
        await page.locator('main').evaluate((n, padding) => { n.style.padding = padding; }, padding);
        await resizeComplete();
        for (const [speed, text, state] of [[0, '0', 'available'], [5, '5', 'available'], [85.49, '85', 'available'],
          [85.5, '86', 'available'], [160, '160', 'available'], [999, '999', 'available'],
          [999.01, 'OVR', 'overflow'], [null, 'X', 'unavailable']]) await reading(speed, state, text);
      }
      await page.locator('main').evaluate(n => { n.style.padding = '3px'; });
      await resizeComplete();
      for (const pitch of [-90, 90]) {
        setFixture({ pitch_deg: pitch });
        await page.waitForFunction(pitch => Math.sign(document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f) ===
          Math.sign(pitch), pitch);
        for (const [speed, state, text] of [[160, 'available', '160'], [null, 'unavailable', 'X'], [1000, 'overflow', 'OVR']]) {
          await reading(speed, state, text);
          await page.screenshot({ path: `.local/pfd-browser/tas-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${pitch}-${text}.png` });
        }
      }
    }
    for (const bad of [null, false, '85', {}, [], -0.01]) {
      await reading(160, 'available', '160');
      await reading(bad, 'unavailable', 'X');
      assert.equal(await page.locator('#airspeed').getAttribute('data-state'), 'available');
      assert.equal(await page.locator('#ground-speed-value').textContent(), '170');
    }
    await reading(160, 'available', '160');
    setFixture({ omit_tas: true });
    await page.locator('#true-airspeed[data-state="unavailable"]').waitFor();
    await reading(160, 'available', '160');
    for (const missing of [{ ias_kt: null }, { ias_kt: 150, gs_kt: null }]) {
      setFixture(missing);
      await page.locator(missing.ias_kt === null ? '#airspeed[data-state="unavailable"]' : '#ground-speed[data-state="unavailable"]').waitFor();
      assert.equal(await page.locator('#true-airspeed-value').textContent(), '160');
    }
    setFixture({ attitude_only: true });
    await page.locator('#true-airspeed[data-state="unavailable"]').waitFor();
    setFixture({ attitude_only: false, ias_kt: 150, gs_kt: 170 });
    await reading(0, 'available', '0');
    setFixture({ hold: true });
    await page.locator('.instrument[data-available="false"]').waitFor();
    assert.equal(await page.locator('.instrument').getAttribute('data-state'), 'stale');
    await page.setViewportSize({ width: 320, height: 480 }); await resizeComplete();
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
    setFixture({ hold: false, tas_kt: 85.5 });
    await page.locator('.instrument[data-available="true"]').waitFor();
    await reading(85.5, 'available', '86');
    assert.ok(await page.evaluate(() => window.tasNodes.every((n, i) => n === document.querySelectorAll('#true-airspeed, #true-airspeed *')[i])));
  }
  setFixture({ ias_kt: 123, gs_kt: 137, tas_kt: 131, pitch_deg: 0, roll_deg: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
  console.log('TAS browser checks passed: source values, local failures, old snapshots, expiry/recovery, stable nodes, glyphs, frames and top-strip clearance.');
}

import assert from 'node:assert/strict';

// Real panel and SVG with only the scheduling clock replaced. No wall-clock flash waits.
const controlledApp = `
import { createPanel } from '/panel.js';
import { createFrameScheduler } from '/frame-scheduler.js';
import { createStatusView } from '/status.js';
import { pfdLayout } from '/layout.js';
import { normalizeOptionalIndications } from '/telemetry/validation.js';
const instrument = document.querySelector('.instrument');
const status = createStatusView(document);
const panel = createPanel(instrument, status);
let time = 0, id = 0;
const timers = new Map(), frames = new Map();
const scheduler = createFrameScheduler({ panel, now: () => time,
  setTimer: (fn, ms) => { const key = ++id; timers.set(key, { fn, at: time + ms }); return key; },
  clearTimer: key => timers.delete(key),
  requestFrame: fn => { const key = ++id; frames.set(key, fn); return key; },
  cancelFrame: key => frames.delete(key),
});
function flush() { const pending = [...frames.values()]; frames.clear(); for (const fn of pending) fn(); }
function resize() { const { width, height } = instrument.querySelector('svg').getBoundingClientRect(); scheduler.resize(pfdLayout(width, height)); flush(); }
new ResizeObserver(resize).observe(instrument); resize();
window.alertHarness = {
  accept(altitude, target, pressure = '29.92 inHg') {
    const sourceStatus = { source: 'demo', transport: 'connected', state: 'live' };
    status.update(sourceStatus);
    scheduler.accept({ ...normalizeOptionalIndications({ altitude_ft: altitude, selected_altitude_ft: target, baro_text: pressure, vertical_speed_fpm: 1500, heading_deg: 359, selected_heading_deg: 1 }),
      attitude: { pitch_deg: 0, roll_deg: 0 }, expiresAt: time + 1000, status: sourceStatus }); flush();
  },
  tick(value) { time = value; for (const [key, timer] of [...timers]) if (timer.at <= time) { timers.delete(key); timer.fn(); } flush(); },
  invalidate(reason) { scheduler.invalidate(reason); },
  reset() { scheduler.invalidate('waiting'); time = 0; },
  pending: () => ({ timers: timers.size, frames: frames.size }),
};
`;

async function reading(page) {
  return page.evaluate(() => {
    const group = document.querySelector('#selected-altitude');
    const value = document.querySelector('#selected-altitude-value');
    const bug = document.querySelector('#selected-altitude-bug');
    return { state: group.dataset.state, alert: group.dataset.alert, flashing: group.dataset.flashing,
      text: value.textContent, color: getComputedStyle(value).fill, visibility: getComputedStyle(value).visibility,
      bugDisplay: getComputedStyle(bug).display, aria: group.getAttribute('aria-label'),
      available: document.querySelector('.instrument').dataset.available };
  });
}

export async function checkSelectedAltitude(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ selected_altitude_ft: 12000, altitude_ft: 10000, hold: false, attitude_only: false, state: 'live', pitch_deg: 0 });
    await page.goto(url + path);
    await page.locator('#selected-altitude[data-state="available"]').waitFor();
    assert.equal((await reading(page)).text, '12000');
    setFixture({ selected_altitude_ft: 'bad' });
    await page.locator('#selected-altitude[data-state="unavailable"]').waitFor();
    assert.equal((await reading(page)).available, 'true');
    assert.equal((await reading(page)).bugDisplay, 'none');
    setFixture({ attitude_only: true });
    await page.locator('#altitude[data-state="unavailable"]').waitFor();
    assert.equal((await reading(page)).text, 'X');
    setFixture({ attitude_only: false });
  }

  const route = handler => handler.fulfill({ contentType: 'text/javascript', body: controlledApp });
  await page.route('**/app.js', route);
  try {
    for (const path of ['/', '/layout.html']) {
      await page.goto(url + path);
      await page.waitForFunction(() => !!window.alertHarness);
      await page.evaluate(() => {
        window.selectedNodes = [...document.querySelectorAll('#selected-altitude *, #selected-altitude-bug')];
      });
      for (const [width, height] of [[320, 240], [320, 480], [568, 240], [326, 246]]) {
        await page.setViewportSize({ width, height });
        await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        for (const [altitude, target] of [[1000, 1100], [1000, 5000], [1000, -120], [-9999, -9999], [99999, 99999], [0, -0.1]]) {
          await page.evaluate(({ altitude, target }) => { window.alertHarness.reset(); window.alertHarness.accept(altitude, target); }, { altitude, target });
          const bounds = await page.evaluate(() => {
            const box = document.querySelector('#selected-altitude rect').getBoundingClientRect();
            const value = document.querySelector('#selected-altitude-value').getBoundingClientRect();
            const icon = document.querySelector('.selected-altitude-symbol').getBoundingClientRect();
            const clip = document.querySelector('#altitude-tape-clip rect').getBBox();
            const bug = document.querySelector('#selected-altitude-bug');
            const bugY = bug.transform.baseVal.consolidate().matrix.f;
            const view = document.querySelector('svg').getBoundingClientRect();
            return { valueContained: value.left > icon.right && value.right <= box.right && value.top >= box.top && value.bottom <= box.bottom,
              panelContained: box.left >= view.left && box.right <= view.right && box.top >= view.top,
              bugY, height: clip.height, nodesRetained: window.selectedNodes.every(n => n.isConnected) };
          });
          assert.ok(bounds.valueContained && bounds.panelContained && bounds.nodesRetained, JSON.stringify({ width, height, target, bounds }));
          // SVG matrices round to single precision; compare well below one CSS pixel.
          assert.ok(Math.abs(bounds.bugY - Math.min(bounds.height - 14.5, Math.max(14.5, bounds.height / 2 + (altitude - target) * 0.4))) < 1e-4,
            JSON.stringify({ width, height, altitude, target, bounds }));
        }
        for (const altitude of [10000, -120]) {
          await page.evaluate(altitude => { window.alertHarness.reset(); window.alertHarness.accept(altitude, altitude); }, altitude);
          const paint = await page.evaluate(() => {
            const bug = document.querySelector('#selected-altitude-bug');
            const matrix = bug.getScreenCTM();
            const hits = [[2, -4], [2, 4], [2, 0]].map(([x, y]) => {
              const point = new DOMPoint(x, y).matrixTransform(matrix);
              return document.elementFromPoint(point.x, point.y) === bug;
            });
            const notch = new DOMPoint(4, 0).matrixTransform(matrix);
            const window = document.querySelector('.altitude-window');
            const sign = document.querySelector('#altitude-sign');
            const signBox = sign.getBoundingClientRect();
            return { hits, notchFilledByReadout: document.elementFromPoint(notch.x, notch.y) === window,
              sameSymbol: bug.getAttribute('d') === document.querySelector('.selected-altitude-symbol').getAttribute('d'),
              clearOfSign: !signBox.width || document.elementFromPoint(signBox.left + signBox.width / 2, signBox.top + signBox.height / 2) !== bug };
          });
          assert.ok(paint.hits.every(Boolean) && paint.clearOfSign && paint.notchFilledByReadout && paint.sameSymbol,
            JSON.stringify({ width, height, altitude, paint }));
          if (width === 320 && height === 240 && path === '/') {
            await page.screenshot({ path: `.local/pfd-browser/selected-altitude-capture-${altitude}.png` });
          }
        }
        await page.evaluate(() => { window.alertHarness.reset(); window.alertHarness.accept(8500, 10000); window.alertHarness.accept(9000, 10000); });
        assert.equal((await reading(page)).alert, 'within-1000');
        assert.equal((await reading(page)).color, 'rgb(134, 222, 245)');
        await page.evaluate(() => window.alertHarness.tick(250));
        assert.equal((await reading(page)).visibility, 'hidden');
        assert.equal((await reading(page)).bugDisplay, 'inline');
        assert.equal((await reading(page)).aria, 'Selected altitude 10000 feet');
        await page.evaluate(() => window.alertHarness.accept(9800, 10000));
        assert.equal((await reading(page)).alert, 'within-200');
        await page.evaluate(() => window.alertHarness.accept(10201, 10000));
        assert.equal((await reading(page)).alert, 'deviation');
        assert.equal((await reading(page)).color, 'rgb(255, 221, 67)');
        await page.screenshot({ path: `.local/pfd-browser/selected-altitude-${path === '/' ? 'production' : 'layout'}-${width}x${height}-deviation.png` });
        for (let time = 750; time <= 5250; time += 500) await page.evaluate(time => {
          window.alertHarness.tick(time); window.alertHarness.accept(10201, 10000);
        }, time);
        assert.equal((await reading(page)).flashing, 'false'); assert.equal((await reading(page)).alert, 'deviation');
        assert.equal((await reading(page)).visibility, 'visible');
        await page.evaluate(() => window.alertHarness.accept(10200, 10000));
        assert.equal((await reading(page)).alert, 'none');
        await page.evaluate(() => window.alertHarness.accept(10201, 10000, null));
        assert.equal((await reading(page)).alert, 'deviation');
        await page.evaluate(() => window.alertHarness.accept(null, 10000));
        assert.equal((await reading(page)).text, '10000'); assert.equal((await reading(page)).alert, 'none');
        assert.equal((await reading(page)).bugDisplay, 'none');
        for (const [target, expected, state] of [[null, 'X', 'unavailable'], ['bad', 'X', 'unavailable'], [-10000, 'OVR', 'overflow'], [100000, 'OVR', 'overflow'], [0, '0', 'available']]) {
          await page.evaluate(target => window.alertHarness.accept(0, target), target);
          const current = await reading(page); assert.equal(current.text, expected); assert.equal(current.state, state);
          assert.equal(current.available, 'true');
        }
        await page.evaluate(() => { window.alertHarness.reset(); window.alertHarness.accept(10000, 10000); window.alertHarness.accept(10201, 10000); window.alertHarness.tick(1000); });
        assert.equal((await reading(page)).available, 'false'); assert.equal((await reading(page)).alert, 'none');
        assert.equal(await page.locator('#heading-value').textContent(), 'X');
        assert.equal(await page.locator('#selected-heading-bug').getAttribute('transform'), null);
        assert.equal(await page.locator('#altitude-trend').getAttribute('d'), null);
        await page.evaluate(() => window.alertHarness.tick(1500));
        assert.equal(await page.locator('#heading-value').textContent(), 'X');
        assert.equal(await page.locator('#altitude-trend').getAttribute('d'), null);
        assert.deepEqual(await page.evaluate(() => window.alertHarness.pending()), { timers: 0, frames: 0 });
        await page.evaluate(() => window.alertHarness.accept(10201, 10000));
        assert.equal((await reading(page)).alert, 'none'); assert.equal((await reading(page)).available, 'true');
        assert.equal(await page.locator('#heading-value').textContent(), '359');
        assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'inline');
      }
    }
  } finally { await page.unroute('**/app.js', route); }
  setFixture({ selected_altitude_ft: 12000, altitude_ft: 10000, hold: false, attitude_only: false, state: 'live', pitch_deg: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

// Optional browser acceptance checks. Requires Playwright and an installed browser.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import { once } from 'node:events';
import { readFileSync } from 'node:fs';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? 'playwright');
const server = spawn(process.execPath, ['tests/browser-fixture-server.mjs'], {
  env: { ...process.env, PFD_FIXTURE_PORT: '0' }, stdio: ['pipe', 'pipe', 'inherit'],
});
let activeClients = 0;
const url = await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error('Fixture server did not start')), 5000);
  server.once('error', reject);
  server.stdout.on('data', chunk => {
    const output = String(chunk);
    const address = output.match(/Browser fixture: (http:\/\/127\.0\.0\.1:\d+)/);
    if (address) { clearTimeout(timeout); resolve(address[1]); }
    for (const match of output.matchAll(/SSE clients: (\d+)/g)) activeClients = Number(match[1]);
  });
});
const errors = [];
let browser;
const setFixture = value => server.stdin.write(`${JSON.stringify(value)}\n`);
// Executed in the page: hit testing includes every ancestor clip.
function backgroundCovered() {
  const svg = document.querySelector('svg');
  const [,, width, height] = svg.getAttribute('viewBox').split(' ').map(Number);
  return [0.01, 0.25, 0.5, 0.75, 0.99].every(x =>
    [0.01, 0.25, 0.5, 0.75, 0.99].every(y => {
      const point = new DOMPoint(x * width, y * height).matrixTransform(svg.getScreenCTM());
      return document.elementsFromPoint(point.x, point.y).some(node => node.matches('.sky, .ground'));
    }));
}
function instrumentNodesUnchanged() {
  const nodes = [...document.querySelectorAll('svg *')];
  return nodes.length === window.initialInstrumentNodes.length &&
    nodes.every((node, index) => node === window.initialInstrumentNodes[index]);
}
try {
  browser = await chromium.launch({ headless: true, channel: process.env.BROWSER_CHANNEL || undefined });
  const page = await browser.newPage();
  await page.addInitScript(() => {
    const Original = window.ResizeObserver;
    window.ResizeObserver = class extends Original {
      constructor(callback) {
        super((entries, observer) => {
          callback(entries, observer);
          if (!window.resizeChecks) return;
          const pitch = document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f;
          const tick = document.querySelector('#pitch-ladder [data-pitch="10"]').getPointAtLength(0).y;
          window.resizeChecks.push(Math.abs(pitch + tick) < 1e-4);
        });
      }
    };
  });
  page.on('pageerror', error => errors.push(String(error)));
  await mkdir('.local/pfd-browser', { recursive: true });
  for (const path of ['/', '/layout.html']) {
    for (const [width, height] of [[320, 240], [326, 246], [320, 480], [390, 664], [568, 240], [667, 280], [844, 320]]) {
      await page.setViewportSize({ width, height });
      await page.goto(url + path);
      await page.locator('.instrument[data-available="true"]').waitFor();
      if (path === '/layout.html') await page.locator('#layout-fixtures[data-items]').waitFor({ state: 'attached' });
      // Flush ResizeObserver geometry and the scheduled sample before capturing paint.
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      assert.equal(await page.locator('header, #data-status, #transport, #source-badge').count(), 0);
      assert.equal(await page.locator('#telemetry-status').textContent().then(text => text.includes('DEMO')), false);
      assert.deepEqual(await page.locator('svg > g').evaluateAll(nodes => nodes.map(n => Number(n.dataset.layer))), [0, 1, 2, 3, 4]);
      const dimensions = await page.evaluate(() => {
        const root = document.documentElement;
        const svg = document.querySelector('svg');
        const rect = svg.getBoundingClientRect();
        const [,, w, h] = svg.getAttribute('viewBox').split(' ').map(Number);
        const ball = document.querySelector('.slip-ball').getBoundingClientRect();
        const triangle = document.querySelector('.bank-zero').getBBox();
        const clipTop = Number(document.querySelector('#attitude-clip rect').getAttribute('y'));
        return { scrollWidth: root.scrollWidth, scrollHeight: root.scrollHeight,
          w, h, width: rect.width, height: rect.height, ballWidth: ball.width, ballHeight: ball.height,
          bankZeroPaintedTop: triangle.y + document.querySelector('#fixed-symbols').transform.baseVal.consolidate().matrix.f - 1, clipTop };
      });
      assert.equal(dimensions.scrollWidth, width);
      assert.equal(dimensions.scrollHeight, height);
      assert.ok(Math.abs(dimensions.w - dimensions.width) < 0.01);
      assert.ok(Math.abs(dimensions.h - dimensions.height) < 0.01);
      assert.ok(Math.abs(dimensions.ballWidth - dimensions.ballHeight) < 0.01);
      assert.ok(dimensions.bankZeroPaintedTop >= dimensions.clipTop);
      if (path === '/layout.html') {
        const collisions = await page.evaluate(() => {
          const failures = [];
          for (const name of ['airspeed', 'altitude']) {
            const group = document.querySelector(`#fixture-${name}`);
            const window = group.querySelector('rect[stroke]');
            const text = window.nextElementSibling.getBBox();
            const rect = window.getBBox();
            // Text stays within the opaque value window, clear of its 1px stroke.
            if (text.x < rect.x + 0.5 || text.x + text.width > rect.x + rect.width - 0.5)
              failures.push(`${name} readout crosses its painted window`);
          }
          // Background overlap is intentional; compare painted label content.
          const badge = document.querySelector('#layout-badge .fixture-label').getBoundingClientRect();
          const gs = [...document.querySelectorAll('#fixture-supplemental text')].find(n => n.textContent.startsWith('GS')).getBoundingClientRect();
          if (badge.bottom >= gs.top) failures.push('Fixture badge collides with ground speed');
          return failures;
        });
        assert.deepEqual(collisions, [], `${width}x${height} painted content`);
      }
      await page.screenshot({ path: `.local/pfd-browser/${path === '/' ? 'g1' : 'layout'}-${width}x${height}.png` });
    }
  }
  // Candidate opacity is a development experiment; mobile evidence selects the final value.
  for (const opacity of [0.50, 0.65, 0.80]) {
    for (const [width, height] of [[320, 480], [568, 240]]) {
      for (const pitch of [-90, 90]) {
        setFixture({ pitch_deg: pitch, roll_deg: 0 });
        await page.setViewportSize({ width, height });
        await page.goto(`${url}/layout.html?opacity=${opacity}`);
        await page.locator('.instrument[data-available="true"]').waitFor();
        await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        assert.equal(await page.locator('#fixture-airspeed .instrument-background').evaluate(n => Number(getComputedStyle(n).fillOpacity)), opacity);
        assert.equal(await page.locator('#fixture-airspeed').evaluate(n => getComputedStyle(n).opacity), '1');
        assert.equal(await page.locator('#fixture-airspeed text').first().evaluate(n => getComputedStyle(n).fillOpacity), '1');
        await page.screenshot({ path: `.local/pfd-browser/opacity-${opacity}-${width}x${height}-${pitch}.png` });
      }
    }
  }
  setFixture({ pitch_deg: 0, roll_deg: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  // Instrument nodes survive samples and orientation changes.
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
  for (const viewport of [{ width: 320, height: 480 }, { width: 844, height: 320 }]) {
    await page.setViewportSize(viewport);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    assert.equal(await page.evaluate(instrumentNodesUnchanged), true);
  }
  assert.equal(await page.evaluate(() =>
    document.querySelector('#world-pitch').parentElement === document.querySelector('#world-rotation')), true);
  // Check real instruments against a pristine detached DOM, without a second live controller.
  assert.equal(await page.evaluate(async () => {
    const { createPanel } = await import('/panel.js');
    const { createStatusView } = await import('/status.js');
    const { pfdLayout } = await import('/layout.js');
    const doc = new DOMParser().parseFromString(await (await fetch('/')).text(), 'text/html');
    const panel = createPanel(doc.querySelector('.instrument'), createStatusView(doc));
    const frame = Object.freeze({ attitude: Object.freeze({ pitch_deg: 10, roll_deg: 25 }),
      slip_skid: 0.5, turn_rate_dps: 3, expiresAt: 1000,
      status: Object.freeze({ source: 'demo', state: 'live', transport: 'connected' }) });
    const original = JSON.stringify(frame);
    for (const layout of [pfdLayout(314, 474), pfdLayout(562, 234)]) {
      panel.resize(layout); panel.render(frame);
    }
    return JSON.stringify(frame) === original;
  }), true);
  // Independent pitch offsets for two actual usable panel sizes; no layout formula copy.
  setFixture({ pitch_deg: 10, roll_deg: 0 });
  await page.waitForFunction(() => {
    const pitch = document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f;
    const tick = document.querySelector('#pitch-ladder [data-pitch="10"]').getPointAtLength(0).y;
    return Math.abs(pitch + tick) < 1e-4 &&
      document.querySelector('#world-rotation').getAttribute('transform').startsWith('rotate(0 ');
  });
  await page.evaluate(() => { window.resizeChecks = []; });
  for (const [width, height, offset] of [[320, 480, 71.6363636364], [568, 240, 32]]) {
    await page.setViewportSize({ width, height });
    await page.waitForFunction(offset => Math.abs(
      document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f - offset) < 1e-4, offset);
    assert.equal(await page.evaluate(instrumentNodesUnchanged), true);
  }
  const resizeChecks = await page.evaluate(() => { const checks = window.resizeChecks; window.resizeChecks = null; return checks; });
  assert.ok(resizeChecks.length >= 2);
  assert.ok(resizeChecks.every(Boolean), 'ResizeObserver must leave a coherent pitch scale and reading');
  const poses = JSON.parse(readFileSync(new URL('./fixtures/pfd-samples.json', import.meta.url)));
  for (const [index, pose] of poses.entries()) {
    setFixture({ ...pose.attitude, slip_skid: pose.slip_skid, turn_rate_dps: pose.turn_rate_dps });
    await page.waitForFunction(({ slip, turn }) =>
      document.querySelector('#slip-skid').getAttribute('aria-label') ===
        (slip === 0 ? 'Slip/skid centered' : `Slip/skid ${slip < 0 ? 'left' : 'right'}`) &&
      document.querySelector('#turn-rate').getAttribute('aria-label') === `Turn rate ${turn} degrees per second`,
      { slip: pose.slip_skid, turn: pose.turn_rate_dps });
    await page.waitForFunction(({ pitch, roll }) => {
      const scale = -document.querySelector('#pitch-ladder [data-pitch="10"]').getPointAtLength(0).y / 10;
      return Math.abs(document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f - pitch * scale) < 1e-4 &&
        document.querySelector('#world-rotation').getAttribute('transform').startsWith(`rotate(${-roll} `);
    }, { pitch: pose.attitude.pitch_deg, roll: pose.attitude.roll_deg });
    const directions = await page.evaluate(() => ({
      ball: Number(document.querySelector('.slip-ball').getAttribute('cx')),
      vector: Number(document.querySelector('.turn-vector').getAttribute('d').split('H ')[1]),
    }));
    assert.equal(Math.sign(directions.ball), Math.sign(pose.slip_skid));
    assert.equal(Math.sign(directions.vector), Math.sign(pose.turn_rate_dps));
    if (Math.abs(pose.turn_rate_dps) === 3) assert.equal(Math.abs(directions.vector), 40);
    const geometry = await page.evaluate(() => {
      const fixed = document.querySelector('#fixed-symbols').getCTM();
      const world = document.querySelector('#world-pitch').getCTM();
      return { cx: fixed.e, cy: fixed.f, x: world.e, y: world.f,
        scale: -document.querySelector('#pitch-ladder [data-pitch="10"]').getPointAtLength(0).y / 10 };
    });
    const bank = pose.attitude.roll_deg * Math.PI / 180;
    const displacement = pose.attitude.pitch_deg * geometry.scale;
    // SVG matrices round to float precision. Allow 0.0001 CSS px, far below painted geometry.
    assert.ok(Math.abs(geometry.x - (geometry.cx + Math.sin(bank) * displacement)) < 1e-4);
    assert.ok(Math.abs(geometry.y - (geometry.cy + Math.cos(bank) * displacement)) < 1e-4,
      JSON.stringify({ index, geometry, expectedY: geometry.cy + Math.cos(bank) * displacement }));
    assert.equal(await page.evaluate(backgroundCovered), true, `Full background coverage for pose ${index}`);
    assert.equal(await page.evaluate(instrumentNodesUnchanged), true);
    await page.screenshot({ path: `.local/pfd-browser/pose-${index}.png` });
  }
  // Prove that the coverage check rejects the regression it is intended to catch.
  const clip = page.locator('#background-clip rect');
  const originalClip = await clip.evaluate(node => Object.fromEntries(
    ['x', 'y', 'width', 'height'].map(key => [key, node.getAttribute(key)])));
  try {
    await clip.evaluate(node => {
      const central = document.querySelector('#attitude-clip rect');
      for (const key of ['x', 'y', 'width', 'height']) node.setAttribute(key, central.getAttribute(key));
    });
    assert.equal(await page.evaluate(backgroundCovered), false, 'Narrowed clip must fail coverage');
  } finally {
    await clip.evaluate((node, values) => {
      for (const [key, value] of Object.entries(values)) node.setAttribute(key, value);
    }, originalClip);
  }
  assert.equal(await page.evaluate(backgroundCovered), true);
  const ticks = await page.locator('#pitch-ladder path').evaluateAll(nodes => nodes.map(n => Number(n.dataset.pitch)));
  assert.deepEqual(ticks, Array.from({ length: 73 }, (_, i) => -90 + i * 2.5).filter(n => n !== 0));
  const chevrons = await page.locator('#pitch-warnings path').evaluateAll(nodes => nodes.map(n => Number(n.dataset.pitch)));
  assert.deepEqual(chevrons, [60, 70, 80, 90, -40, -50, -60, -70, -80, -90]);
  setFixture({ pitch_deg: 0, roll_deg: 0, slip_skid: 'bad', turn_rate_dps: -3 });
  await page.locator('#slip-skid[data-available="false"]').waitFor();
  assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
  assert.equal(await page.locator('#turn-rate').getAttribute('data-available'), 'true');
  setFixture({ slip_skid: 1, turn_rate_dps: 'bad' });
  await page.locator('#turn-rate[data-available="false"]').waitFor();
  assert.equal(await page.locator('#slip-skid').getAttribute('data-available'), 'true');
  setFixture({ attitude_only: true });
  await page.locator('#slip-skid[data-available="false"]').waitFor();
  assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
  setFixture({ attitude_only: false, slip_skid: 0, turn_rate_dps: 9 });
  await page.waitForFunction(() => document.querySelector('.turn-overflow').getAttribute('d') !== '');
  for (const value of [90, -90]) {
    setFixture({ pitch_deg: value, roll_deg: value > 0 ? 179.999 : -180 });
    await page.waitForFunction(value => {
      const scale = -document.querySelector('#pitch-ladder [data-pitch="10"]').getPointAtLength(0).y / 10;
      return Math.abs(document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f - value * scale) < 1e-4;
    }, value);
    assert.equal(await page.evaluate(backgroundCovered), true, `Full background coverage at pitch ${value}`);
    await page.screenshot({ path: `.local/pfd-browser/extreme-${value}.png` });
  }
  setFixture({ hold: true });
  await page.locator('.instrument[data-available="false"]').waitFor();
  assert.equal(await page.locator('.instrument').getAttribute('data-state'), 'stale');
  assert.equal(await page.locator('.unavailable-panel').evaluate(n => {
    const cover = n.getBoundingClientRect(), panel = n.parentElement.getBoundingClientRect();
    return cover.x === panel.x && cover.y === panel.y && cover.width === panel.width &&
      cover.height === panel.height && getComputedStyle(n).backgroundColor === 'rgb(16, 21, 27)';
  }), true);
  // Simulate space consumed by cutouts/browser bars; actual phones remain separate evidence.
  await page.evaluate(() => { document.querySelector('main').style.padding = '15px 24px 12px 12px'; });
  await page.setViewportSize({ width: 320, height: 480 });
  assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  setFixture({ hold: false, pitch_deg: 10, roll_deg: 25, turn_rate_dps: 3 });
  await page.locator('.instrument[data-available="true"]').waitFor();
  for (const state of ['paused', 'disconnected', 'invalid', 'waiting', 'stale']) {
    setFixture({ state });
    await page.waitForFunction(state => document.querySelector('.instrument').dataset.state === state, state);
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  }
  setFixture({ state: 'live', malformed: true });
  await page.waitForFunction(() => document.querySelector('.instrument').dataset.state === 'invalid');
  setFixture({ malformed: false });
  await page.locator('.instrument[data-available="true"]').waitFor();
  // Exercise the actual page lifecycle handlers and their subscription ownership.
  await page.evaluate(() => window.dispatchEvent(new Event('pagehide')));
  assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  await page.evaluate(() => { window.dispatchEvent(new Event('pageshow')); window.dispatchEvent(new Event('pageshow')); });
  await page.locator('.instrument[data-available="true"]').waitFor();
  assert.equal(activeClients, 1);
  assert.deepEqual(errors, []);
  console.log(`PFD browser checks passed on ${await browser.version()}. Viewports, fixtures, poses, optional failures, stale, resize and lifecycle checked.`);
} finally {
  if (browser) await browser.close();
  server.stdin.end();
  server.kill();
  if (server.exitCode === null) await once(server, 'exit');
}

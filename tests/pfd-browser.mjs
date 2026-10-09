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
try {
  browser = await chromium.launch({ headless: true, channel: process.env.BROWSER_CHANNEL || undefined });
  const page = await browser.newPage();
  page.on('pageerror', error => errors.push(String(error)));
  await mkdir('.local/pfd-browser', { recursive: true });
  for (const path of ['/', '/layout.html']) {
    for (const [width, height] of [[320, 480], [390, 664], [568, 240], [667, 280], [844, 320]]) {
      await page.setViewportSize({ width, height });
      await page.goto(url + path);
      await page.locator('.instrument[data-available="true"]').waitFor();
      if (path === '/layout.html') await page.locator('#layout-fixtures[data-items]').waitFor();
      // Flush ResizeObserver geometry and the scheduled sample before capturing paint.
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
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
          bankZeroPaintedTop: triangle.y - 1, clipTop };
      });
      assert.equal(dimensions.scrollWidth, width);
      assert.equal(dimensions.scrollHeight, height);
      assert.ok(Math.abs(dimensions.w - dimensions.width) < 0.01);
      assert.ok(Math.abs(dimensions.h - dimensions.height) < 0.01);
      assert.ok(Math.abs(dimensions.ballWidth - dimensions.ballHeight) < 0.01);
      assert.ok(dimensions.bankZeroPaintedTop >= dimensions.clipTop);
      await page.screenshot({ path: `.local/pfd-browser/${path === '/' ? 'g1' : 'layout'}-${width}x${height}.png` });
    }
  }
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  const poses = JSON.parse(readFileSync(new URL('./fixtures/pfd-samples.json', import.meta.url)));
  for (const [index, pose] of poses.entries()) {
    setFixture({ ...pose.attitude, slip_skid: pose.slip_skid, turn_rate_dps: pose.turn_rate_dps });
    await page.waitForFunction(({ slip, turn }) =>
      document.querySelector('#slip-skid').getAttribute('aria-label') ===
        (slip === 0 ? 'Slip/skid centered' : `Slip/skid ${slip < 0 ? 'left' : 'right'}`) &&
      document.querySelector('#turn-rate').getAttribute('aria-label') === `Turn rate ${turn} degrees per second`,
      { slip: pose.slip_skid, turn: pose.turn_rate_dps });
    await page.waitForFunction(({ pitch, roll }) => {
      const svg = document.querySelector('svg');
      const h = Number(svg.getAttribute('viewBox').split(' ')[3]);
      const scale = Math.max(3.2, (h - 80) / 55);
      return document.querySelector('#world-pitch').getAttribute('transform') === `translate(0 ${pitch * scale})` &&
        document.querySelector('#world-rotation').getAttribute('transform').startsWith(`rotate(${-roll} `);
    }, { pitch: pose.attitude.pitch_deg, roll: pose.attitude.roll_deg });
    const directions = await page.evaluate(() => ({
      ball: Number(document.querySelector('.slip-ball').getAttribute('cx')),
      vector: Number(document.querySelector('.turn-vector').getAttribute('d').split('H ')[1]),
    }));
    assert.equal(Math.sign(directions.ball), Math.sign(pose.slip_skid));
    assert.equal(Math.sign(directions.vector), Math.sign(pose.turn_rate_dps));
    if (Math.abs(pose.turn_rate_dps) === 3) assert.equal(Math.abs(directions.vector), 40);
    await page.screenshot({ path: `.local/pfd-browser/pose-${index}.png` });
  }
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
      const h = Number(document.querySelector('svg').getAttribute('viewBox').split(' ')[3]);
      const scale = Math.max(3.2, (h - 80) / 55);
      return document.querySelector('#world-pitch').getAttribute('transform') === `translate(0 ${value * scale})`;
    }, value);
    await page.screenshot({ path: `.local/pfd-browser/extreme-${value}.png` });
  }
  setFixture({ hold: true });
  await page.locator('.instrument[data-available="false"]').waitFor();
  assert.equal(await page.locator('#data-status').getAttribute('data-state'), 'stale');
  // Simulate space consumed by cutouts/browser bars; actual phones remain separate evidence.
  await page.evaluate(() => { document.querySelector('main').style.padding = '15px 24px 12px 12px'; });
  await page.setViewportSize({ width: 320, height: 480 });
  assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  setFixture({ hold: false, pitch_deg: 10, roll_deg: 25, turn_rate_dps: 3 });
  await page.locator('.instrument[data-available="true"]').waitFor();
  for (const state of ['paused', 'disconnected', 'invalid', 'waiting', 'stale']) {
    setFixture({ state });
    await page.waitForFunction(state => document.querySelector('#data-status').dataset.state === state, state);
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  }
  setFixture({ state: 'live', malformed: true });
  await page.waitForFunction(() => document.querySelector('#data-status').dataset.state === 'invalid');
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

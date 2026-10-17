// Optional smoke test against the compiled Windows application, including a real restart.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { once } from 'node:events';
import { setTimeout as delay } from 'node:timers/promises';
import { resolve } from 'node:path';
import { mkdir } from 'node:fs/promises';

const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright');
const reservation = createServer();
reservation.listen(0, '127.0.0.1');
await once(reservation, 'listening');
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const address = `http://127.0.0.1:${port}`;
const executable = resolve(process.argv[2] ?? 'target/x86_64-pc-windows-msvc/debug/msfs2024-artificial-horizon.exe');
let server;
let browser;
async function start() {
  server = spawn(executable, ['--source', 'demo', '--port', String(port)], { stdio: ['ignore', 'pipe', 'pipe'] });
  let failure;
  server.once('error', error => { failure = error; });
  const deadline = performance.now() + 5000;
  while (performance.now() < deadline) {
    if (failure) throw failure;
    if (server.exitCode !== null) throw new Error('Rust server exited before startup');
    try {
      const response = await fetch(`${address}/health`, { signal: AbortSignal.timeout(500) });
      if (response.ok) { assert.deepEqual(await response.json(), { status: 'ok' }); return; }
    } catch { /* Retry only the bounded startup probe. */ }
    await delay(25);
  }
  throw new Error('Rust server did not become healthy');
}
async function stop() {
  if (server?.pid && server.exitCode === null) {
    const exited = once(server, 'exit');
    server.kill();
    await exited;
  }
}
async function currentSnapshot() {
  const response = await fetch(`${address}/api/v1/events`, { signal: AbortSignal.timeout(2000) });
  const reader = response.body.getReader();
  let wire = '';
  while (!wire.includes('\n\n')) wire += new TextDecoder().decode((await reader.read()).value);
  await reader.cancel();
  return JSON.parse(wire.split('\n').find(line => line.startsWith('data: ')).slice(6));
}
const altitudeReading = page => page.evaluate(() => ({
  state: document.querySelector('#altitude').dataset.state,
  feet: Number(document.querySelector('#altitude').getAttribute('aria-label').split(' ')[2]),
  sign: document.querySelector('#altitude-sign').textContent,
  drums: Number([...document.querySelectorAll('#altitude-digits > g')].map(n => n.firstElementChild.textContent).join('')),
  baroState: document.querySelector('#barometric-setting').dataset.state,
  pressureLabel: document.querySelector('#barometric-setting').getAttribute('aria-label'),
  pressureText: document.querySelector('#barometric-value').textContent,
}));
function verifyAltitude(reading) {
  assert.equal(reading.state, 'available');
  assert.equal(reading.baroState, 'available');
  assert.ok(Number.isFinite(reading.feet));
  assert.equal(reading.drums, Math.floor(Math.abs(reading.feet) / 20) * 20);
  assert.equal(reading.sign, reading.feet < 0 ? '−' : '');
  assert.match(reading.pressureText, /^\d+\.\d{2} inHg$/);
  assert.equal(reading.pressureLabel, `Barometric setting ${reading.pressureText}`);
}

async function checkInitialDisplay(page) {
  await page.locator('.instrument[data-available="true"]').waitFor();
  assert.equal(await page.locator('#source-badge, .source').count(), 0);
  assert.equal(await page.locator('#telemetry-status').textContent().then(text => text.includes('DEMO')), false);
  assert.equal(await page.locator('#slip-skid').getAttribute('data-available'), 'true');
  assert.equal(await page.locator('#turn-rate').getAttribute('data-available'), 'true');
  verifyAltitude(await altitudeReading(page));
}

async function checkEmbeddedAssets(loadedModules) {
  assert.deepEqual([...loadedModules].sort(), [
    '/airspeed/ground-speed.js', '/airspeed/true-airspeed.js', '/airspeed/helpers.js', '/airspeed/ias.js', '/airspeed/index.js',
    '/airspeed/readout.js', '/airspeed/references.js', '/airspeed/tape.js', '/altitude/baro.js', '/altitude/index.js', '/altitude/readout.js', '/altitude/selected.js', '/altitude/tape.js', '/altitude/trend.js', '/altitude/vsi.js', '/app.js', '/fixed-symbols/aircraft.js', '/fixed-symbols/bank-scale.js', '/fixed-symbols/index.js', '/frame-scheduler.js', '/heading.js', '/helpers.js', '/horizon/bank-pointer.js', '/horizon/index.js', '/horizon/pitch-scale.js', '/horizon/world.js', '/layout.js',
    '/panel.js', '/slip-skid.js', '/status.js', '/telemetry/client.js', '/telemetry/validation.js', '/turn-rate.js',
  ].sort());
  for (const path of ['/altitude/vsi.js', '/altitude/trend.js', '/heading.js', '/airspeed/true-airspeed.js']) {
    const response = await fetch(address + path, { method: 'HEAD' });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('content-type'), 'text/javascript; charset=utf-8');
    assert.equal(await response.text(), '');
  }
  for (const path of ['/math.js', '/svg.js', '/telemetry-client.js', '/airspeed.js', '/horizon.js', '/fixed-symbols.js', '/airspeed/calculations.js', '/horizon/calculations.js', '/vertical-speed.js']) {
    assert.equal((await fetch(address + path)).status, 404);
  }
}

async function checkInitialTelemetry(page) {
  // The first synthetic-jump sample intentionally omits VSI for one tick.
  await page.locator('#vertical-speed[data-state="available"]').waitFor();
  const snapshot = await currentSnapshot();
  assert.equal(snapshot.schema_version, 1);
  assert.equal(snapshot.source, 'demo');
  assert.equal(snapshot.state, 'live');
  assert.ok(Number.isFinite(snapshot.slip_skid));
  assert.ok(Number.isFinite(snapshot.turn_rate_dps));
  assert.ok(Number.isFinite(snapshot.ias_kt));
  assert.ok(Number.isFinite(snapshot.gs_kt));
  assert.ok(Number.isFinite(snapshot.tas_kt));
  assert.ok(Number.isFinite(snapshot.altitude_ft));
  assert.match(snapshot.baro_text, /^\d+\.\d{2} inHg$/);
  assert.ok(Number.isFinite(snapshot.vertical_speed_fpm));
  assert.equal(snapshot.heading_deg, 354);
  assert.ok(Number.isFinite(snapshot.selected_heading_deg));
  assert.equal(await page.locator('#heading-value').textContent(), '354');
  await page.waitForFunction(() => document.querySelector('#vertical-speed-value').textContent === '+4881');
  assert.equal(await page.locator('#vertical-speed-value').textContent(), '+4881');
  assert.ok(Number.isFinite(snapshot.ias_acceleration_ktps));
  assert.equal(snapshot.airspeed_profile.vne_kt, 140);
}

async function checkApproachingOverspeed(page) {
  // Observe source-supplied acceleration and the corresponding six-second warning.
  await page.waitForFunction(() => document.querySelector('#airspeed').dataset.approaching === 'true' &&
    Number(document.querySelector('#airspeed-trend').dataset.predictionKt) >= 140,
    null, { timeout: 25000 });
  assert.equal(await page.locator('#airspeed').getAttribute('data-state'), 'available');
  assert.equal(await page.locator('#ground-speed').getAttribute('data-state'), 'available');
}

async function checkRightTurnAndSpeeds(page) {
  // Wait for the Rust-owned right-bank/standard-rate segment, not a browser fixture.
  await page.waitForFunction(() => document.querySelector('#turn-rate').getAttribute('aria-label') ===
    'Turn rate 3 degrees per second' &&
    document.querySelector('#world-rotation').getAttribute('transform').startsWith('rotate(-30 '),
    null, { timeout: 25000 });
  const speeds = await page.evaluate(() => ({
    ias: document.querySelector('#airspeed').getAttribute('aria-label'),
    digits: [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.firstElementChild.textContent).join(''),
    gs: document.querySelector('#ground-speed-value').textContent,
    tas: document.querySelector('#true-airspeed-value').textContent,
    tasLabel: document.querySelector('#true-airspeed').getAttribute('aria-label'),
  }));
  assert.deepEqual(speeds, { ias: 'Indicated airspeed 150 knots', digits: '150', gs: '170', tas: '160', tasLabel: 'True airspeed 160 knots' });
  assert.equal(await page.locator('#airspeed').getAttribute('data-overspeed'), 'true');
  assert.equal(await page.locator('#airspeed-trend').evaluate(n => getComputedStyle(n).display), 'none');
  await page.screenshot({ path: '.local/pfd-runtime-landscape.png' });
}

async function checkNorthCrossing(page) {
  // The source crosses north after the +3 degrees/second segment.
  await page.waitForFunction(() => document.querySelector('#heading').getAttribute('aria-label') ===
    'Magnetic heading 0 degrees, selected heading 0 degrees', null, { timeout: 20000 });
  const north = await page.evaluate(() => ({
    value: document.querySelector('#heading-value').textContent,
    center: document.querySelector('#heading-tape-clip rect').getBBox().width / 2,
    bug: document.querySelector('#selected-heading-bug').transform.baseVal.consolidate().matrix.e,
  }));
  assert.equal(north.value, '360'); assert.ok(Math.abs(north.center - north.bug) < 1e-4);
  await page.screenshot({ path: '.local/pfd-runtime-heading-north.png' });
}

async function checkSelectedAltitude(page) {
  await page.locator('#selected-altitude[data-state="available"]').waitFor({ timeout: 20000 });
  const target = await currentSnapshot();
  assert.equal(target.selected_altitude_ft, 10000);
  assert.equal(target.altitude_ft, 8500);
  assert.equal(await page.locator('#selected-altitude-value').textContent(), '10000');
  assert.equal(await page.locator('#selected-altitude').getAttribute('aria-label'), 'Selected altitude 10000 feet');
  assert.equal(await page.locator('#selected-altitude').getAttribute('data-alert'), 'none');
  assert.equal(await page.locator('#selected-altitude-bug').evaluate(n => n.transform.baseVal.consolidate().matrix.f), 14.5);
  await page.screenshot({ path: '.local/pfd-runtime-selected-altitude.png' });
}

async function checkAltitudeHold(page) {
  await page.waitForFunction(() => document.querySelector('#vertical-speed').getAttribute('aria-label') === 'Vertical speed 0 feet per minute');
  const held = await currentSnapshot();
  assert.equal(held.altitude_ft, 8500); assert.ok(held.vertical_speed_fpm === 0);
  assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'none');
}

async function checkClimbAndTrend(page) {
  await page.waitForFunction(() => document.querySelector('#vertical-speed').getAttribute('aria-label') === 'Vertical speed 1500 feet per minute', null, { timeout: 10000 });
  const climbing = await currentSnapshot();
  assert.ok(climbing.altitude_ft >= 8500 && climbing.altitude_ft < 8600);
  assert.equal(climbing.vertical_speed_fpm, 1500);
  assert.equal(await page.locator('#vertical-speed-value').textContent(), '+1500');
  const altitudeTrend = await page.locator('#altitude-trend').evaluate(node => {
    const start = node.getPointAtLength(0), end = node.getPointAtLength(node.getTotalLength());
    const altitude = Number(document.querySelector('#altitude').getAttribute('aria-label').split(' ')[2]);
    const point = new DOMPoint(start.x, (start.y + end.y) / 2).matrixTransform(node.getScreenCTM());
    return { x: start.x, dy: end.y - start.y, predictionChange: Number(node.dataset.predictionFt) - altitude,
      painted: document.elementFromPoint(point.x, point.y) === node };
  });
  assert.deepEqual(altitudeTrend, { x: 2, dy: -60, predictionChange: 150, painted: true });
  await page.screenshot({ path: '.local/pfd-runtime-vertical-speed.png' });
}

async function checkSourceLoss(page) {
  await page.locator('.instrument[data-available="false"]').waitFor();
  assert.equal(await page.locator('#heading-value').textContent(), 'X');
  assert.equal(await page.locator('#selected-heading-bug').getAttribute('transform'), null);
  assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'unavailable');
  assert.equal(await page.locator('#altitude-trend').evaluate(n => getComputedStyle(n).display), 'none');
}

async function checkRecovery(page) {
  await page.locator('.instrument[data-available="true"]').waitFor();
  assert.equal(await page.locator('#turn-rate').getAttribute('aria-label'), 'Turn rate 0 degrees per second');
  const recovered = await page.evaluate(() => ({
    state: document.querySelector('#airspeed').dataset.state,
    ias: Number(document.querySelector('#airspeed').getAttribute('aria-label').split(' ')[2]),
    digits: [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.firstElementChild.textContent).join(''),
    gsState: document.querySelector('#ground-speed').dataset.state,
    tas: Number(document.querySelector('#true-airspeed').getAttribute('aria-label').split(' ')[2]),
    tasText: document.querySelector('#true-airspeed-value').textContent,
  }));
  assert.equal(recovered.state, 'available');
  assert.equal(recovered.gsState, 'available');
  assert.equal(await page.locator('#true-airspeed').getAttribute('data-state'), 'available');
  // Reconnect can happen after the initial zero-speed segment.
  assert.equal(recovered.tas, Math.round(recovered.ias * 160 / 150));
  assert.equal(recovered.tasText, String(recovered.tas));
  assert.equal(recovered.digits, String(Math.floor(recovered.ias)));
  verifyAltitude(await altitudeReading(page));
  assert.equal(await page.locator('#heading-value').textContent(), '354');
  assert.equal(await page.locator('#selected-altitude').getAttribute('data-state'), 'unavailable');
  await page.locator('#vertical-speed[data-state="available"]').waitFor();
  assert.equal(await page.locator('#vertical-speed-value').textContent(), '+4881');
}

try {
  await mkdir('.local', { recursive: true });
  await start();
  browser = await chromium.launch({ headless: true, channel: process.env.BROWSER_CHANNEL || undefined });
  const page = await browser.newPage({ viewport: { width: 568, height: 240 } });
  const errors = [];
  const loadedModules = new Set();
  page.on('pageerror', error => errors.push(String(error)));
  page.on('response', response => {
    const path = new URL(response.url()).pathname;
    if (path.endsWith('.js')) {
      if (response.status() !== 200 || !response.headers()['content-type']?.startsWith('text/javascript')) {
        errors.push(`Module response: ${path} (${response.status()})`);
      }
      loadedModules.add(path);
    }
  });
  await page.goto(address);
  await checkInitialDisplay(page);
  await checkEmbeddedAssets(loadedModules);
  await checkInitialTelemetry(page);
  await checkApproachingOverspeed(page);
  await checkRightTurnAndSpeeds(page);
  await checkNorthCrossing(page);
  await checkSelectedAltitude(page);
  await checkAltitudeHold(page);
  await checkClimbAndTrend(page);
  await stop();
  await checkSourceLoss(page);
  await start();
  await checkRecovery(page);
  assert.deepEqual(errors, []);
  console.log(`Rust-to-browser PFD checks passed on ${await browser.version()}: embedded assets, heading/north capture, selected-altitude and VSI SSE/SVG, standard-rate turn, source loss and restart without page reload.`);
} finally {
  if (browser) await browser.close();
  await stop();
}

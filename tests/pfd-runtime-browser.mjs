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
const executable = resolve(process.argv[2] ?? 'target/x86_64-pc-windows-msvc/debug/main.exe');
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
  await page.locator('.instrument[data-available="true"]').waitFor();
  assert.equal(await page.locator('#source-badge, .source').count(), 0);
  assert.equal(await page.locator('#telemetry-status').textContent().then(text => text.includes('DEMO')), false);
  assert.equal(await page.locator('#slip-skid').getAttribute('data-available'), 'true');
  assert.equal(await page.locator('#turn-rate').getAttribute('data-available'), 'true');
  assert.deepEqual([...loadedModules].sort(), [
    '/airspeed.js', '/app.js', '/fixed-symbols.js', '/frame-scheduler.js', '/horizon.js', '/layout.js',
    '/panel.js', '/slip-skid.js', '/status.js', '/svg.js', '/telemetry-client.js', '/turn-rate.js',
  ]);
  const response = await fetch(`${address}/api/v1/events`, { signal: AbortSignal.timeout(2000) });
  const reader = response.body.getReader();
  let wire = '';
  while (!wire.includes('\n\n')) wire += new TextDecoder().decode((await reader.read()).value);
  await reader.cancel();
  const snapshot = JSON.parse(wire.split('\n').find(line => line.startsWith('data: ')).slice(6));
  assert.equal(snapshot.schema_version, 1);
  assert.equal(snapshot.source, 'demo');
  assert.equal(snapshot.state, 'live');
  assert.ok(Number.isFinite(snapshot.slip_skid));
  assert.ok(Number.isFinite(snapshot.turn_rate_dps));
  assert.ok(Number.isFinite(snapshot.ias_kt));
  assert.ok(Number.isFinite(snapshot.gs_kt));
  assert.equal(await page.locator('#airspeed').getAttribute('data-state'), 'available');
  assert.equal(await page.locator('#ground-speed').getAttribute('data-state'), 'available');
  // Wait for the Rust-owned right-bank/standard-rate segment, not a browser fixture.
  await page.waitForFunction(() => document.querySelector('#turn-rate').getAttribute('aria-label') ===
    'Turn rate 3 degrees per second' &&
    document.querySelector('#world-rotation').getAttribute('transform').startsWith('rotate(-30 '),
    null, { timeout: 25000 });
  const speeds = await page.evaluate(() => ({
    ias: document.querySelector('#airspeed').getAttribute('aria-label'),
    digits: [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.firstElementChild.textContent).join(''),
    gs: document.querySelector('#ground-speed-value').textContent,
  }));
  assert.deepEqual(speeds, { ias: 'Indicated airspeed 150 knots', digits: '150', gs: '170' });
  await page.screenshot({ path: '.local/pfd-runtime-landscape.png' });
  await stop();
  await page.locator('.instrument[data-available="false"]').waitFor();
  await start();
  await page.locator('.instrument[data-available="true"]').waitFor();
  assert.equal(await page.locator('#turn-rate').getAttribute('aria-label'), 'Turn rate 0 degrees per second');
  const recovered = await page.evaluate(() => ({
    state: document.querySelector('#airspeed').dataset.state,
    ias: Number(document.querySelector('#airspeed').getAttribute('aria-label').split(' ')[2]),
    digits: [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.firstElementChild.textContent).join(''),
    gsState: document.querySelector('#ground-speed').dataset.state,
  }));
  assert.equal(recovered.state, 'available');
  assert.equal(recovered.gsState, 'available');
  assert.equal(recovered.digits, String(Math.floor(recovered.ias)));
  assert.deepEqual(errors, []);
  console.log(`Rust-to-browser PFD checks passed on ${await browser.version()}: embedded assets, extended SSE, standard-rate turn, source loss and restart without page reload.`);
} finally {
  if (browser) await browser.close();
  await stop();
}

import { test } from 'node:test';
import assert from 'node:assert/strict';
import net from 'node:net';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { launchOwned } from './process.mjs';

const exe = process.env.S1_EXE;
async function reserve() {
  let connections = 0;
  const server = net.createServer(socket => { connections++; socket.destroy(); });
  server.connectionCount = () => connections;
  await new Promise((resolve, reject) => server.listen(0, '127.0.0.1', resolve).once('error', reject));
  return server;
}

test('process test fixtures are supplied', () => {
  assert.ok(exe, 'Set S1_EXE to the absolute path of the rebuilt application');
  assert.ok(process.env.S1_IGNORE_EXE, 'Set S1_IGNORE_EXE to the compiled ignore-shutdown fixture');
  assert.ok(process.env.S1_OUTPUT_EXE, 'Set S1_OUTPUT_EXE to the compiled output fixture');
});

test('rejects an absent executable', async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  try {
    await assert.rejects(launchOwned({ exe: path.join(dir, 'absent.exe'), source: 'demo', runDirectory: dir }));
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('never accepts or stops an unrelated port owner', { skip: !exe }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  const server = await reserve();
  const port = server.address().port;
  try {
    await assert.rejects(launchOwned({ exe, source: 'demo', port, runDirectory: dir, readyTimeoutMs: 5000 }));
    assert.equal(server.listening, true);
    assert.equal(server.connectionCount(), 0);
  } finally { server.close(); await rm(dir, { recursive: true, force: true }); }
});

test('owns a Demo run and stops it idempotently', { skip: !exe }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  let app;
  try {
    app = await launchOwned({ exe, source: 'demo', runDirectory: dir, readyTimeoutMs: 10000 });
    assert.equal(app.artifact.before.sha256.length, 64);
    assert.ok(Number.isFinite(Date.parse(app.startedAt)));
    assert.ok(app.invocation.args.includes('--source'));
    assert.equal(app.artifact.attribution, 'external/unverified');
    const response = await fetch(`${app.endpoint}/health`);
    assert.equal(response.ok, true);
    const first = await app.stop();
    assert.equal(first.graceful, true);
    assert.deepEqual(await app.stop(), first);
  } finally { await app?.stop(); await rm(dir, { recursive: true, force: true }); }
});

test('rejects cancellation before startup', async () => {
  const controller = new AbortController();
  controller.abort();
  await assert.rejects(launchOwned({ exe: path.resolve('missing.exe'), source: 'demo', runDirectory: path.resolve('.') }, controller.signal), /Cancelled/);
});

test('repeats Demo with an active SSE subscription and releases the port', { skip: !exe }, async () => {
  for (let i = 0; i < 2; i++) {
    const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
    try {
      const app = await launchOwned({ exe, source: 'demo', runDirectory: dir });
      const response = await fetch(`${app.endpoint}/api/v1/events`);
      const reader = response.body.getReader();
      const result = await reader.read();
      assert.match(new TextDecoder().decode(result.value), /event: telemetry/);
      await reader.cancel();
      assert.equal((await app.stop()).graceful, true);
      const port = Number(new URL(app.endpoint).port);
      const server = net.createServer();
      await new Promise((resolve, reject) => server.once('error', reject).listen(port, '127.0.0.1', resolve));
      await new Promise(resolve => server.close(resolve));
    } finally { await rm(dir, { recursive: true, force: true }); }
  }
});

test('forces only an owned child that ignores shutdown', { skip: !process.env.S1_IGNORE_EXE }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  try {
    const app = await launchOwned({ exe: process.env.S1_IGNORE_EXE, source: 'demo', runDirectory: dir, shutdownTimeoutMs: 100 });
    const result = await app.stop();
    assert.equal(result.forced, true);
    assert.equal(result.graceful, false);
    assert.deepEqual(await app.stop(), result);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('cancellation after startup stops the owned child', { skip: !exe }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  const controller = new AbortController();
  try {
    const app = await launchOwned({ exe, source: 'demo', runDirectory: dir }, controller.signal);
    const response = await fetch(`${app.endpoint}/api/v1/events`);
    await response.body.cancel();
    controller.abort();
    assert.equal((await app.stop()).graceful, true);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('cancellation during startup cleans up the owned child', { skip: !exe }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  const controller = new AbortController();
  try {
    const launch = launchOwned({ exe, source: 'demo', runDirectory: dir }, controller.signal);
    setTimeout(() => controller.abort(), 50);
    await assert.rejects(launch, error => error.message === 'Cancelled' && error.cleanup?.graceful === true);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('does not report an already exited child as a successful shutdown request', { skip: !exe }, async () => {
  const dir = await mkdtemp(path.join(tmpdir(), 'live process '));
  try {
    const app = await launchOwned({ exe, source: 'demo', runDirectory: dir });
    process.kill(app.pid);
    for (let i = 0; i < 100; i++) {
      try { await fetch(`${app.endpoint}/health`); await new Promise(resolve => setTimeout(resolve, 50)); }
      catch { break; }
    }
    assert.equal((await app.stop()).graceful, false);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('successful stop leaves no shutdown deadline keeping the harness alive', { skip: !exe }, async () => {
  const { execFile } = await import('node:child_process');
  const { promisify } = await import('node:util');
  await promisify(execFile)(process.execPath, [path.resolve('tools/live/process-run.fixture.mjs')], { timeout: 6000, env: process.env });
});

test('cancellation during artifact hashing does not spawn the executable', { skip: !exe }, async () => {
  const { access } = await import('node:fs/promises');
  const dir = await mkdtemp(path.join(tmpdir(), 'live hash '));
  try {
    const controller = new AbortController();
    let observed = false;
    const launch = launchOwned({ exe, source: 'demo', runDirectory: dir, testHooks: {
      onFirstHashChunk() { observed = true; controller.abort(); },
    } }, controller.signal);
    try { const app = await launch; await app.stop(); assert.fail('launch was not cancelled'); }
    catch (error) { assert.match(error.message, /Cancelled/); }
    assert.equal(observed, true);
    await assert.rejects(access(path.join(dir, 'stdout.txt')), { code: 'ENOENT' });
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('stop waits for complete stdout and stderr files', { skip: !process.env.S1_OUTPUT_EXE }, async () => {
  const { stat } = await import('node:fs/promises');
  const dir = await mkdtemp(path.join(tmpdir(), 'live output '));
  try {
    const app = await launchOwned({ exe: process.env.S1_OUTPUT_EXE, source: 'demo', runDirectory: dir });
    assert.equal((await app.stop()).graceful, true);
    assert.equal((await stat(path.join(dir, 'stdout.txt'))).size, 8 * 1024 * 1024);
    assert.equal((await stat(path.join(dir, 'stderr.txt'))).size, 8 * 1024 * 1024);
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('exit waiting has a separate bounded deadline', async () => {
  const { waitForExit } = await import('./process.mjs');
  assert.equal(typeof waitForExit, 'function');
  assert.equal(await waitForExit(new Promise(() => {}), 50), false);
  assert.equal(await waitForExit(Promise.resolve(), 50), true);
});

test('detects executable changes between prelaunch and postlaunch hashes', { skip: !exe }, async () => {
  const { copyFile, open } = await import('node:fs/promises');
  const dir = await mkdtemp(path.join(tmpdir(), 'live identity '));
  try {
    const copy = path.join(dir, 'changing.exe');
    await copyFile(exe, copy);
    let changed = false;
    const app = await launchOwned({ exe: copy, source: 'demo', runDirectory: dir, testHooks: {
      async afterFirstHash() {
        const file = await open(copy, 'r+');
        const byte = Buffer.alloc(1);
        await file.read(byte, 0, 1, 0x40);
        await file.write(Buffer.from([byte[0] ^ 1]), 0, 1, 0x40);
        await file.close();
        changed = true;
      },
    } });
    try {
      assert.equal(changed, true);
      assert.equal(app.artifact.changed, true);
      assert.notEqual(app.artifact.before.sha256, app.artifact.after.sha256);
    } finally { assert.equal((await app.stop()).graceful, true); }
  } finally { await rm(dir, { recursive: true, force: true }); }
});

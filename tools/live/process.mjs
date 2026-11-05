import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { stat, writeFile } from 'node:fs/promises';
import net from 'node:net';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const OWNER_SCRIPT = '$ErrorActionPreference="Stop"; $p=[int]$env:LIVE_PORT; @(Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -eq $p -and $_.LocalAddress -eq "127.0.0.1" } | ForEach-Object { [int]$_.OwningProcess }) | ConvertTo-Json -Compress';
const STREAM_CAP = 8 * 1024 * 1024;

export function applicationArgs({ source, dll, runDirectory, logPath }, port) {
  if (logPath && source !== 'simconnect') throw new Error('Diagnostics require SimConnect');
  if (logPath && !path.isAbsolute(logPath)) throw new Error('Diagnostic path must be absolute');
  const args = ['--source', source, '--listen-address', '127.0.0.1', '--port', String(port), '--shutdown-file', path.join(runDirectory, 'shutdown')];
  if (dll) args.push('--dll', dll);
  if (logPath) args.push('--log', logPath);
  return args;
}

async function identity(exe, onFirstChunk) {
  const info = await stat(exe);
  if (!info.isFile()) throw new Error('Executable is not a file');
  const hash = createHash('sha256');
  let first = true;
  for await (const chunk of createReadStream(exe)) {
    hash.update(chunk);
    if (first) { first = false; await onFirstChunk?.(); }
  }
  return { sha256: hash.digest('hex'), size: info.size, mtimeMs: info.mtimeMs };
}

async function selectPort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => server.once('error', reject).listen(0, '127.0.0.1', resolve));
  const port = server.address().port;
  await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
  return port;
}

export function capture(stream, filename, onLimit) {
  const file = createWriteStream(filename);
  let count = 0;
  let truncated = false;
  let error = null;
  const done = new Promise(resolve => file.once('close', resolve));
  file.on('error', e => { error = e.message; });
  stream.on('data', chunk => {
    const remaining = Math.max(0, STREAM_CAP - count);
    if (remaining) file.write(chunk.subarray(0, remaining));
    count += chunk.length;
    if (count > STREAM_CAP && !truncated) { truncated = true; onLimit?.(); }
  });
  stream.on('end', () => file.end());
  return { done, result: () => ({ truncated, error }) };
}

function owner(port, timeoutMs) {
  return new Promise((resolve, reject) => {
    const child = spawn('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', OWNER_SCRIPT], {
      env: { ...process.env, LIVE_PORT: String(port) }, windowsHide: true,
    });
    let out = '', err = '';
    child.stdout.on('data', chunk => { out += chunk; });
    child.stderr.on('data', chunk => { err += chunk; });
    const timer = setTimeout(() => child.kill(), timeoutMs);
    child.once('error', error => { clearTimeout(timer); reject(error); });
    child.once('close', code => {
      clearTimeout(timer);
      if (code !== 0) return reject(new Error(`Port ownership inspection failed: ${err.trim()}`));
      try { const value = JSON.parse(out || '[]'); resolve(Array.isArray(value) ? value : [value]); }
      catch (error) { reject(new Error(`Invalid port ownership result: ${error.message}`)); }
    });
  });
}

export async function waitForExit(exited, timeoutMs) {
  let timer;
  const result = await Promise.race([
    exited.then(() => true),
    new Promise(resolve => { timer = setTimeout(() => resolve(false), timeoutMs); }),
  ]);
  clearTimeout(timer);
  return result;
}

export async function launchOwned(options, signal) {
  const { exe, source, dll, runDirectory } = options;
  if (!path.isAbsolute(exe) || !path.isAbsolute(runDirectory)) throw new Error('Executable and run directory must be absolute paths');
  if (!['demo', 'simconnect'].includes(source)) throw new Error('Source must be demo or simconnect');
  if (dll && (source !== 'simconnect' || !path.isAbsolute(dll))) throw new Error('DLL requires simconnect and an absolute path');
  if (signal?.aborted) throw new Error('Cancelled');
  const before = await identity(exe, options.testHooks?.onFirstHashChunk);
  await options.testHooks?.afterFirstHash?.();
  if (signal?.aborted) throw new Error('Cancelled');
  const port = options.port ?? await selectPort();
  if (signal?.aborted) throw new Error('Cancelled');
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid port');
  const shutdownPath = path.join(runDirectory, 'shutdown');
  const args = applicationArgs(options, port);
  const startedAt = new Date().toISOString();
  const child = spawn(exe, args, { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
  const stdout = capture(child.stdout, path.join(runDirectory, 'stdout.txt'), options.onOutputLimit);
  const stderr = capture(child.stderr, path.join(runDirectory, 'stderr.txt'), options.onOutputLimit);
  let exit = null;
  const exited = new Promise(resolve => {
    child.once('error', error => resolve(exit = { code: null, error: error.message }));
    child.once('exit', (code, sig) => resolve(exit = { code, signal: sig }));
  });
  let stopped;
  const stop = () => stopped ??= (async () => {
    let error = null, forced = false;
    let requested = false;
    if (!exit) {
      try { await writeFile(shutdownPath, '', { flag: 'wx' }); requested = true; }
      catch (e) { error = `Shutdown request failed: ${e.message}`; }
      await waitForExit(exited, options.shutdownTimeoutMs ?? 10000);
      if (!exit) {
        forced = true;
        if (!child.kill()) error = 'Failed to terminate owned child';
        if (!await waitForExit(exited, options.forceTimeoutMs ?? 5000)) {
          error = 'Owned child exit was not confirmed after termination';
        }
      }
    }
    if (exit && !await waitForExit(Promise.all([stdout.done, stderr.done]), options.forceTimeoutMs ?? 5000)) {
      error = 'Output stream completion was not confirmed';
    }
    return { graceful: requested && !forced && !error && exit?.code === 0, forced, exitCode: exit?.code ?? null, error: error ?? exit?.error ?? null, stdout: stdout.result(), stderr: stderr.result() };
  })();
  const abort = () => { void stop(); };
  signal?.addEventListener('abort', abort, { once: true });
  if (signal?.aborted) abort();
  try {
    const after = await identity(exe);
    const artifact = { path: exe, before, after, changed: JSON.stringify(before) !== JSON.stringify(after), attribution: 'external/unverified' };
    const deadline = Date.now() + (options.readyTimeoutMs ?? 30000);
    let verified = false;
    while (Date.now() < deadline) {
      if (signal?.aborted) throw new Error('Cancelled');
      if (exit) throw new Error(`Application exited during startup: ${JSON.stringify(exit)}`);
      const pids = await owner(port, options.ownershipTimeoutMs ?? 5000);
      if (pids.some(pid => pid !== child.pid)) throw new Error('Port belongs to another process');
      if (pids.includes(child.pid)) { verified = true; break; }
      await delay(50);
    }
    if (!verified) throw new Error('Timed out waiting for owned endpoint');
    const endpoint = `http://127.0.0.1:${port}`;
    const response = await fetch(`${endpoint}/health`, { signal: AbortSignal.timeout(options.httpTimeoutMs ?? 10000) });
    if (!response.ok) throw new Error(`Health check failed: ${response.status}`);
    const verifiedAgain = await owner(port, options.ownershipTimeoutMs ?? 5000);
    if (verifiedAgain.length !== 1 || verifiedAgain[0] !== child.pid) throw new Error('Port ownership changed during startup');
    if (signal?.aborted || exit) throw new Error('Application stopped during startup');
    return { pid: child.pid, startedAt, invocation: { executable: exe, args }, endpoint, artifact, shutdownPath, stop };
  } catch (error) {
    const cleanup = await stop();
    error.cleanup = cleanup;
    throw error;
  }
}

import path from 'node:path';
import {open} from 'node:fs/promises';
import {finished} from 'node:stream/promises';
import {pathToFileURL} from 'node:url';
import {launchOwned} from './process.mjs';
import {capture} from './observe.mjs';

export function parseArgs(args) {
  const values = {}, names = new Set(['exe','url','output','duration-seconds','max-messages']);
  let i = 0;
  for (; i < args.length && args[i] !== '--'; i += 2) {
    const option = args[i]; const key = option?.startsWith('--') ? option.slice(2) : '';
    if (!names.has(key) || values[key] !== undefined || !args[i+1] || args[i+1].startsWith('--')) throw new Error(`Invalid option ${option}`);
    values[key] = args[i+1];
  }
  for (const key of ['exe','url','output']) if (!values[key]) throw new Error(`Missing --${key}`);
  let url; try { url = new URL(values.url); } catch { throw new Error('Invalid URL'); }
  if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || url.username || url.password) throw new Error('URL must use HTTP on 127.0.0.1');
  const number = (key, max) => { const value=values[key]; if (value === undefined) return undefined; if (!/^\d+$/.test(value) || !Number.isSafeInteger(+value) || +value < 1 || +value > max) throw new Error(`Invalid --${key}`); return +value; };
  const durationSeconds = number('duration-seconds',2147483), maxMessages = number('max-messages',Number.MAX_SAFE_INTEGER);
  if (!durationSeconds && !maxMessages) throw new Error('A measurement limit is required');
  return {exe:values.exe,url:values.url,output:values.output,...(durationSeconds ? {durationSeconds} : {}),...(maxMessages ? {maxMessages} : {}),appArgs:args.slice(i + (args[i] === '--' ? 1 : 0))};
}
export async function run(options) {
  let file, stream, app, result, failure, cancelled = false;
  const controller = new AbortController();
  const cancel = () => {cancelled = true; controller.abort();};
  process.on('SIGINT', cancel);
  const startup = new AbortController(); let timeout;
  try {
    file = await open(options.output, 'wx'); stream = file.createWriteStream();
    timeout = setTimeout(() => startup.abort(), 10000);
    app = await launchOwned({...options,exe:path.resolve(options.exe)}, AbortSignal.any([controller.signal,startup.signal]));
    result = await capture({...options,output:stream,signal:controller.signal,startupSignal:startup.signal});
    if (result.httpStatus >= 400) throw new Error(`HTTP ${result.httpStatus}`);
  } catch (error) { failure = error; if (error.cleanup && (error.cleanup.forced || error.cleanup.error || error.cleanup.exitCode !== 0)) failure = new Error(`Cleanup failed: ${error.cleanup.error ?? (error.cleanup.forced ? 'forced termination' : error.cleanup.exitCode)}`); }
  finally {
    clearTimeout(timeout);
    if (stream) { stream.end(); try { await finished(stream); } catch (error) { failure = error; } }
    else if (file) await file.close();
    if (app) { const cleanup = await app.stop(); if (cleanup.forced || cleanup.error || cleanup.exitCode !== 0) failure = new Error(`Cleanup failed: ${cleanup.error ?? (cleanup.forced ? 'forced termination' : cleanup.exitCode)}`); }
    process.removeListener('SIGINT', cancel);
  }
  if (failure && !(cancelled && failure.name === 'AbortError')) { console.error(failure.message); return 1; }
  if (cancelled) return 130;
  console.log(`Captured ${result.bytes} bytes (${result.reason})`); return 0;
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) { try { process.exitCode = await run(parseArgs(process.argv.slice(2))); } catch (error) { console.error(error.message); process.exitCode = 1; } }

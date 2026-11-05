import { once } from 'node:events';
import http from 'node:http';

export class SseMessageCounter {
  constructor() { this.prefix = ''; this.lineLength = 0; this.hasData = false; this.afterCR = false; }
  push(bytes) {
    let count = 0;
    const finishLine = () => {
      if (this.lineLength === 0) { if (this.hasData) count++; this.hasData = false; }
      else if (this.prefix === 'data' || this.prefix.startsWith('data:')) this.hasData = true;
      this.prefix = ''; this.lineLength = 0;
    };
    for (const b of bytes) {
      if (this.afterCR) { this.afterCR = false; if (b === 10) continue; }
      if (b === 13 || b === 10) { finishLine(); this.afterCR = b === 13; }
      else { this.lineLength++; if (this.prefix.length < 5) this.prefix += String.fromCharCode(b); }
    }
    return count;
  }
}

export async function capture({url, output, durationSeconds, maxMessages, signal, startupSignal}) {
  const controller = new AbortController();
  let reason, bytes = 0, messages = 0, timer, outputError;
  const onOutputError = error => { outputError = error; controller.abort(); };
  output.on('error', onOutputError);
  const cancel = () => { reason ??= 'cancelled'; controller.abort(); };
  const startupAbort = () => controller.abort();
  signal?.addEventListener('abort', cancel, {once:true});
  startupSignal?.addEventListener('abort', startupAbort, {once:true});
  if (signal?.aborted) cancel();
  if (startupSignal?.aborted) startupAbort();
  try {
    const response = await new Promise((resolve, reject) => {
      const request = http.get(url, {signal:controller.signal}, resolve);
      request.once('error', reject);
    });
    startupSignal?.removeEventListener('abort', startupAbort);
    if (durationSeconds) timer = setTimeout(() => { reason = 'duration'; controller.abort(); }, durationSeconds * 1000);
    const counter = maxMessages ? new SseMessageCounter() : null;
    try {
      for await (const chunk of response) {
        if (!output.write(chunk)) await once(output, 'drain');
        bytes += chunk.length;
        if (counter) { messages += counter.push(chunk); if (messages >= maxMessages) { reason = 'max-messages'; controller.abort(); break; } }
      }
    } catch (error) { if (outputError) throw outputError; if (!reason) throw error; }
    if (outputError) throw outputError;
    return {bytes, ...(counter ? {messages} : {}), reason:reason ?? 'eof', httpStatus:response.statusCode};
  } finally {
    clearTimeout(timer);
    output.removeListener('error', onOutputError);
    signal?.removeEventListener('abort', cancel);
    startupSignal?.removeEventListener('abort', startupAbort);
  }
}

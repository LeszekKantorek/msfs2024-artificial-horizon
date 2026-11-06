import { once } from 'node:events';
import http from 'node:http';
import {setTimeout as delay} from 'node:timers/promises';

export class SseDataParser {
  constructor() { this.decoder = new TextDecoder(); this.line = ''; this.data = []; this.afterCR = false; }
  push(bytes) {
    const records = [];
    const finishLine = () => {
      if (!this.line) { if (this.data.length) records.push(this.data.join(' ')); this.data = []; }
      else if (this.line === 'data') this.data.push('');
      else if (this.line.startsWith('data:')) { let value = this.line.slice(5); if (value.startsWith(' ')) value = value.slice(1); this.data.push(value); }
      this.line = '';
    };
    for (const char of this.decoder.decode(bytes, {stream:true})) {
      if (this.afterCR) { this.afterCR = false; if (char === '\n') continue; }
      if (char === '\r' || char === '\n') { finishLine(); this.afterCR = char === '\r'; }
      else this.line += char;
    }
    return records;
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
    let response;
    while (!response) {
      try {
        response = await new Promise((resolve, reject) => {
          const request = http.get(url, {signal:controller.signal}, resolve);
          request.once('error', reject);
        });
      } catch (error) {
        if (error.code !== 'ECONNREFUSED' || controller.signal.aborted) throw error;
        await delay(100, undefined, {signal:controller.signal});
      }
    }
    startupSignal?.removeEventListener('abort', startupAbort);
    if (response.statusCode >= 400) {
      response.destroy();
      return {bytes, messages, reason:'eof', httpStatus:response.statusCode};
    }
    if (durationSeconds) timer = setTimeout(() => { reason = 'duration'; controller.abort(); }, durationSeconds * 1000);
    const parser = new SseDataParser();
    try {
      for await (const chunk of response) {
        const records = parser.push(chunk);
        for (const record of records) { const line = `${record}\n`; if (!output.write(line)) await once(output, 'drain'); bytes += Buffer.byteLength(line); messages++; }
        if (maxMessages && messages >= maxMessages) { reason = 'max-messages'; controller.abort(); break; }
      }
    } catch (error) { if (outputError) throw outputError; if (!reason) throw error; }
    if (outputError) throw outputError;
    return {bytes, messages, reason:reason ?? 'eof', httpStatus:response.statusCode};
  } finally {
    clearTimeout(timer);
    output.removeListener('error', onOutputError);
    signal?.removeEventListener('abort', cancel);
    startupSignal?.removeEventListener('abort', startupAbort);
  }
}

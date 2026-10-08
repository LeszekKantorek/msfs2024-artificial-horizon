// Development-only fixture transport. No test routes or controls enter the release binary.
// Run with Node 24; send JSON lines on stdin to change state, attitude or hold samples.
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';

let fixture = { state: 'live', pitch_deg: 0, roll_deg: 0, hold: false };
let sequence = 0;
let clients = 0;
const input = createInterface({ input: process.stdin });
input.on('line', line => {
  try { fixture = { ...fixture, ...JSON.parse(line) }; console.log('Fixture:', fixture); }
  catch { console.error('Expected a JSON object'); }
});
const types = { '/': 'text/html', '/styles.css': 'text/css', '/app.js': 'text/javascript',
  '/telemetry-client.js': 'text/javascript', '/horizon.js': 'text/javascript' };
const server = createServer((request, response) => {
  if (request.url === '/api/v1/events') {
    response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
    response.write('retry: 2000\n\n');
    clients++; console.log('SSE clients:', clients);
    const timer = setInterval(() => {
      if (fixture.hold) { response.write(': heartbeat\n\n'); return; }
      if (fixture.malformed) { response.write('event: telemetry\ndata: broken JSON\n\n'); return; }
      const snapshot = { schema_version: 1, sequence: sequence++, source: 'demo',
        state: fixture.state, sample_age_ms: fixture.sample_age_ms ?? 0,
        attitude: fixture.state === 'live' ? { pitch_deg: fixture.pitch_deg, roll_deg: fixture.roll_deg } : null };
      response.write(`event: telemetry\ndata: ${JSON.stringify(snapshot)}\n\n`);
    }, 50);
    response.on('close', () => { clearInterval(timer); clients--; console.log('SSE clients:', clients); });
    return;
  }
  if (!Object.hasOwn(types, request.url)) { response.writeHead(404).end(); return; }
  const file = request.url === '/' ? 'index.html' : request.url.slice(1);
  response.writeHead(200, { 'Content-Type': `${types[request.url]}; charset=utf-8` });
  response.end(readFileSync(new URL(`../web/${file}`, import.meta.url)));
});
server.listen(8082, '127.0.0.1', () => console.log('Browser fixture: http://127.0.0.1:8082'));
process.on('SIGINT', () => { input.close(); server.close(); server.closeAllConnections(); });

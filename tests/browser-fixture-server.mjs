// Development-only fixture transport. No test routes or controls enter the release binary.
// Run with Node 24; send JSON lines on stdin to change state, attitude or hold samples.
import { createServer } from 'node:http';
import { isIP } from 'node:net';
import { readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';

const profile = JSON.parse(readFileSync(new URL('./fixtures/airspeed-profile.json', import.meta.url)));
let fixture = { state: 'live', pitch_deg: 0, roll_deg: 0, slip_skid: 0, turn_rate_dps: 0,
  heading_deg: 359, selected_heading_deg: 1,
  altitude_ft: 10000, selected_altitude_ft: 12000, vertical_speed_fpm: 0, baro_text: '29.92 inHg', ias_kt: 123, gs_kt: 137, airspeed_profile: profile, ias_acceleration_ktps: 5, hold: false };
let sequence = 0;
let clients = 0;
const input = createInterface({ input: process.stdin });
input.on('line', line => {
  try { fixture = { ...fixture, ...JSON.parse(line) }; console.log('Fixture:', fixture); }
  catch { console.error('Expected a JSON object'); }
});
const types = {
  '/heading.js': 'text/javascript',
  '/altitude/trend.js': 'text/javascript',
  '/altitude/vsi.js': 'text/javascript',
  '/altitude/selected.js': 'text/javascript',
  '/altitude/index.js': 'text/javascript', '/altitude/tape.js': 'text/javascript',
  '/altitude/readout.js': 'text/javascript', '/altitude/baro.js': 'text/javascript', '/': 'text/html', '/styles.css': 'text/css', '/app.js': 'text/javascript',
  '/telemetry/client.js': 'text/javascript', '/horizon/index.js': 'text/javascript',
  '/horizon/world.js': 'text/javascript',
  '/horizon/pitch-scale.js': 'text/javascript',
  '/horizon/bank-pointer.js': 'text/javascript',
  '/telemetry/validation.js': 'text/javascript',
  '/airspeed/index.js': 'text/javascript', '/airspeed/ias.js': 'text/javascript', '/airspeed/ground-speed.js': 'text/javascript', '/airspeed/tape.js': 'text/javascript', '/airspeed/readout.js': 'text/javascript', '/airspeed/references.js': 'text/javascript', '/airspeed/helpers.js': 'text/javascript',
  '/helpers.js': 'text/javascript',
  '/layout.js': 'text/javascript', '/panel.js': 'text/javascript', '/frame-scheduler.js': 'text/javascript', '/fixed-symbols/index.js': 'text/javascript',
  '/fixed-symbols/bank-scale.js': 'text/javascript',
  '/fixed-symbols/aircraft.js': 'text/javascript', '/slip-skid.js': 'text/javascript', '/turn-rate.js': 'text/javascript', '/status.js': 'text/javascript',
  '/layout.html': 'text/html', '/layout-fixture.js': 'text/javascript' };
const server = createServer((request, response) => {
  const path = new URL(request.url, 'http://127.0.0.1').pathname;
  if (path === '/api/v1/events') {
    response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
    response.write('retry: 2000\n\n');
    clients++; console.log('SSE clients:', clients);
    const timer = setInterval(() => {
      if (fixture.hold) { response.write(': heartbeat\n\n'); return; }
      if (fixture.malformed) { response.write('event: telemetry\ndata: broken JSON\n\n'); return; }
      const snapshot = { schema_version: 1, sequence: sequence++, source: 'demo',
        state: fixture.state, sample_age_ms: fixture.sample_age_ms ?? 0,
        attitude: fixture.state === 'live' ? { pitch_deg: fixture.pitch_deg, roll_deg: fixture.roll_deg } : null };
      if (fixture.state === 'live' && !fixture.attitude_only) {
        snapshot.heading_deg = fixture.heading_deg;
        snapshot.selected_heading_deg = fixture.selected_heading_deg;
        snapshot.slip_skid = fixture.slip_skid;
        snapshot.turn_rate_dps = fixture.turn_rate_dps;
        snapshot.altitude_ft = fixture.altitude_ft;
        snapshot.selected_altitude_ft = fixture.selected_altitude_ft;
        snapshot.vertical_speed_fpm = fixture.vertical_speed_fpm;
        if (!fixture.omit_baro) snapshot.baro_text = fixture.baro_text;
        snapshot.ias_kt = fixture.ias_kt;
        snapshot.gs_kt = fixture.gs_kt;
        snapshot.airspeed_profile = fixture.airspeed_profile;
        snapshot.ias_acceleration_ktps = fixture.ias_acceleration_ktps;
      }
      response.write(`event: telemetry\ndata: ${JSON.stringify(snapshot)}\n\n`);
    }, 50);
    response.on('close', () => { clearInterval(timer); clients--; console.log('SSE clients:', clients); });
    return;
  }
  if (!Object.hasOwn(types, path)) { response.writeHead(404).end(); return; }
  const file = path === '/' || path === '/layout.html' ? 'index.html' : path.slice(1);
  response.writeHead(200, { 'Content-Type': `${types[path]}; charset=utf-8` });
  if (path === '/layout-fixture.js') {
    response.end(readFileSync(new URL('./layout-fixture.js', import.meta.url))); return;
  }
  let body = readFileSync(new URL(`../web/${file}`, import.meta.url), 'utf8');
  if (path === '/layout.html') body = body.replace('</head>', '<script type="module" src="/layout-fixture.js"></script></head>');
  response.end(body);
});
const host = process.env.PFD_FIXTURE_HOST ?? '127.0.0.1';
if (!isIP(host)) throw new Error('PFD_FIXTURE_HOST must be an IP address');
server.listen(Number(process.env.PFD_FIXTURE_PORT ?? 8082), host, () =>
  console.log(`Browser fixture: http://${isIP(host) === 6 ? `[${host}]` : host}:${server.address().port}`));
process.on('SIGINT', () => { input.close(); server.close(); server.closeAllConnections(); });

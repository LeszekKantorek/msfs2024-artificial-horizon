import assert from 'node:assert/strict';

const app = `
import { createPanel } from '/panel.js';
import { createFrameScheduler } from '/frame-scheduler.js';
import { createStatusView } from '/status.js';
import { pfdLayout } from '/layout.js';
import { normalizeOptionalIndications } from '/telemetry/validation.js';
const instrument = document.querySelector('.instrument'), status = createStatusView(document);
const panel = createPanel(instrument, status);
let time = 0, id = 0; const frames = new Map();
const scheduler = createFrameScheduler({ panel, now: () => time,
 requestFrame: fn => { frames.set(++id, fn); return id; }, cancelFrame: id => frames.delete(id),
 setTimer: () => ++id, clearTimer() {} });
const flush = () => { const all = [...frames.values()]; frames.clear(); all.forEach(fn => fn()); };
const rect = instrument.querySelector('svg').getBoundingClientRect(); scheduler.resize(pfdLayout(rect.width, rect.height));
window.motion = {
 accept(values, deadline = 2000) {
  const sourceStatus = { source: 'demo', transport: 'connected', state: 'live' }; status.update(sourceStatus);
  scheduler.accept({ ...normalizeOptionalIndications(values), attitude: { pitch_deg: values.pitch, roll_deg: values.roll }, expiresAt: deadline, status: sourceStatus });
 },
 draw(at) { time = at; flush(); }, invalidate(reason) { scheduler.invalidate(reason); },
};
`;

export async function checkSmoothing(page, url) {
  const route = handler => handler.fulfill({ contentType: 'text/javascript', body: app });
  await page.route('**/app.js', route);
  try {
    await page.goto(url); await page.waitForFunction(() => !!window.motion);
    const first = { pitch: 0, roll: 179, heading_deg: 359, selected_heading_deg: 1,
      ias_kt: 100, ias_acceleration_ktps: 1, airspeed_profile: { vne_kt: 120 }, altitude_ft: 10000,
      selected_altitude_ft: 10100, vertical_speed_fpm: 1000, turn_rate_dps: 0, slip_skid: 0 };
    const second = { ...first, pitch: 30, roll: -179, heading_deg: 1, selected_heading_deg: 359,
      ias_kt: 130, altitude_ft: 10100, vertical_speed_fpm: 2000 };
    await page.evaluate(({ first, second }) => { motion.accept(first); motion.draw(0); motion.draw(330); motion.accept(second); motion.draw(330); }, { first, second });
    let reading = await page.evaluate(() => ({ pitch: document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f,
      warning: document.querySelector('#airspeed').dataset.overspeed, label: document.querySelector('#heading').getAttribute('aria-label') }));
    assert.equal(reading.pitch, 0); assert.equal(reading.warning, 'true'); assert.match(reading.label, /heading 1 degrees/);
    await page.evaluate(() => motion.draw(495));
    const geometry = () => {
      const pitch = document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f;
      const scalePitch = document.querySelector('#scale-pitch').transform.baseVal.consolidate().matrix.f;
      const clip = document.querySelector('#airspeed-tape-clip rect').getBBox();
      const altitudeHeight = document.querySelector('#altitude-tape-clip rect').getBBox().height;
      const vector = document.querySelector('#altitude-trend');
      const start = vector.getPointAtLength(0), end = vector.getPointAtLength(vector.getTotalLength());
      return { pitch, scalePitch, worldRoll: document.querySelector('#world-rotation').getAttribute('transform'),
        scaleRoll: document.querySelector('#scale-rotation').getAttribute('transform'),
        ias: (document.querySelector('#airspeed-scale').transform.baseVal.consolidate().matrix.f - clip.height / 2) / 4,
        iasDigits: [...document.querySelectorAll('#airspeed-digits > g')].map(n => n.firstElementChild.textContent).join(''),
        altitudeDigits: [...document.querySelectorAll('#altitude-digits > g')].map(n => n.firstElementChild.textContent).join(''),
        bugY: document.querySelector('#selected-altitude-bug').transform.baseVal.consolidate().matrix.f,
        altitudeCenter: altitudeHeight / 2,
        heading: document.querySelector('#heading-value').textContent,
        headingBug: document.querySelector('#selected-heading-bug').transform.baseVal.consolidate().matrix.e,
        headingCenter: document.querySelector('#heading-tape-clip rect').getBBox().width / 2,
        vectorDelta: end.y - start.y, prediction: vector.getAttribute('data-prediction-ft') };
    };
    reading = await page.evaluate(geometry);
    assert.ok(reading.pitch > 0); assert.equal(reading.pitch, reading.scalePitch);
    assert.equal(reading.worldRoll, reading.scaleRoll); assert.match(reading.worldRoll, /rotate\(180 /);
    assert.equal(reading.heading, '360'); assert.equal(reading.headingBug, reading.headingCenter);
    assert.equal(reading.ias, 115); assert.equal(reading.iasDigits, '115');
    assert.equal(Number(reading.altitudeDigits), 10040); assert.equal(reading.bugY, reading.altitudeCenter - 20);
    assert.equal(reading.vectorDelta, -60); assert.equal(reading.prediction, '10300');
    await page.evaluate(() => motion.draw(660));
    const end = await page.evaluate(geometry);
    assert.ok(Math.abs(end.pitch - reading.pitch * 2) < 1e-4);
    assert.equal(end.ias, 130); assert.equal(end.iasDigits, '130');
    assert.equal(Number(end.altitudeDigits), 10100); assert.equal(end.bugY, end.altitudeCenter);
    assert.equal(end.vectorDelta, -Math.min(80, end.altitudeCenter - 1));
    await page.evaluate(() => { motion.invalidate('paused'); motion.accept({ pitch: -10, roll: 0 }, 3000); motion.draw(700); });
    assert.ok(await page.evaluate(() => document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f < 0));
    await page.evaluate(() => { motion.accept({ pitch: 10, roll: 0 }, 750); motion.draw(750); });
    assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'false');
  } finally { await page.unroute('**/app.js', route); }
}

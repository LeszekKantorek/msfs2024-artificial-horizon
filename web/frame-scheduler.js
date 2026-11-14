import { speedState } from './airspeed/helpers.js';
import { speedIndications } from './airspeed/ias.js';
import { altitudeState } from './altitude/readout.js';
import { altitudePrediction } from './altitude/trend.js';
import { DISPLAY_LIMIT } from './turn-rate.js';

const finite = Number.isFinite;
const heading = v => finite(v) && v >= 0 && v < 360;
const channels = {
  pitch_deg: { get: f => f.attitude?.pitch_deg, attitude: true, valid: finite },
  roll_deg: { get: f => f.attitude?.roll_deg, attitude: true, valid: finite, wrap: 'roll' },
  heading_deg: { valid: heading, wrap: 'heading' },
  selected_heading_deg: { valid: (v, f) => heading(v) && heading(f.heading_deg), wrap: 'heading' },
  ias_kt: { valid: v => speedState(v) === 'available' },
  ias_acceleration_ktps: { valid: (v, f) => speedIndications(f.ias_kt, v, f.airspeed_profile).prediction !== null },
  altitude_ft: { valid: v => altitudeState(v) === 'available' },
  selected_altitude_ft: { valid: (v, f) => altitudeState(v) === 'available' && altitudeState(f.altitude_ft) === 'available' },
  vertical_speed_fpm: { valid: (v, f) => altitudePrediction(f.altitude_ft, v) !== null },
  slip_skid: { valid: v => finite(v) && Math.abs(v) <= 1 },
  turn_rate_dps: { valid: finite, get: f => finite(f.turn_rate_dps) ? Math.max(-DISPLAY_LIMIT, Math.min(DISPLAY_LIMIT, f.turn_rate_dps)) : f.turn_rate_dps },
};
const norm = (v, kind) => kind === 'heading' ? ((v % 360) + 360) % 360 : kind === 'roll' ? ((v + 180) % 360 + 360) % 360 - 180 : v;
function interpolate(from, to, progress, kind) {
  if (progress <= 0) return from;
  if (progress >= 1) return to;
  if (kind) return norm(from + (((to - from + 540) % 360) - 180) * progress, kind);
  return from * (1 - progress) + to * progress;
}

export function createFrameScheduler({ panel, now = () => performance.now(),
  requestFrame = fn => requestAnimationFrame(fn), cancelFrame = id => cancelAnimationFrame(id),
  setTimer = (fn, ms) => setTimeout(fn, ms), clearTimer = id => clearTimeout(id) }) {
  let latest = null, pending = null, pendingLayout = null, generation = 0, timer = null, timerGeneration = 0;
  let previousReceipt = null, baseline = null, baselineDrawTime = null, states = {};
  function cancelTimer() {
    timerGeneration++;
    if (timer !== null) clearTimer(timer);
    timer = null;
  }
  function applyLayout() {
    if (!pendingLayout) return;
    const layout = pendingLayout;
    pendingLayout = null;
    panel.resize(layout);
  }
  function invalidate(reason) {
    generation++;
    cancelTimer();
    if (pending !== null) cancelFrame(pending);
    pending = null;
    latest = null;
    previousReceipt = null;
    baseline = null;
    baselineDrawTime = null;
    states = {};
    panel.invalidate(reason);
    applyLayout();
  }
  function schedule() {
    if (pending !== null || !latest) return;
    const current = generation;
    pending = requestFrame(() => {
      if (current !== generation) return;
      pending = null;
      if (!latest) return;
      const time = now();
      if (time >= latest.expiresAt) { invalidate('stale'); return; }
      applyLayout();
      cancelTimer();
      const display = { ...latest, ...(latest.attitude ? { attitude: { ...latest.attitude } } : {}) };
      let moving = false;
      for (const [key, spec] of Object.entries(channels)) {
        const state = states[key];
        if (!state || !state.valid) continue;
        if (baseline && time >= baseline.expiresAt) { state.value = state.target; state.pending = state.segment = null; }
        const rawTurnOverflow = key === 'turn_rate_dps' && Math.abs(latest.turn_rate_dps) > DISPLAY_LIMIT;
        if (rawTurnOverflow || state.snap) { state.value = state.target; state.segment = null; state.pending = null; state.snap = false; }
        else if (baseline && time < baseline.expiresAt && state.baseValid) {
          state.value = state.baseValue; state.segment = null;
        } else if (state.pending) {
          const target = state.pending;
          if (!state.segment || target.value !== state.segment.to) {
            if (target.value === state.value) state.segment = null;
            else state.segment = { from: state.value, to: target.value,
              start: state.segment ? state.paintTime : Math.max(target.time, baselineDrawTime ?? -Infinity), duration: target.duration };
          }
          state.pending = null;
        }
        if (state.segment) {
          const progress = Math.max(0, Math.min(1, (time - state.segment.start) / state.segment.duration));
          state.value = interpolate(state.segment.from, state.segment.to, progress, spec.wrap);
          if (progress === 1) state.segment = null;
        }
        if (spec.attitude) display.attitude[key] = state.value; else display[key] = state.value;
        state.paintTime = time;
        if (state.segment || state.pending) moving = true;
      }
      if (baseline) { baseline = null; baselineDrawTime = time; } else baselineDrawTime = null;
      const nextUpdateAt = panel.render(display, time, latest);
      if (moving) schedule();
      else if (Number.isFinite(nextUpdateAt)) {
        const token = timerGeneration;
        timer = setTimer(() => { if (current !== generation || token !== timerGeneration) return; timer = null; schedule(); },
          Math.ceil(Math.max(0, Math.min(nextUpdateAt, latest.expiresAt) - now())));
      }
    });
  }
  return {
    accept(frame) {
      const time = now();
      if (time >= frame.expiresAt) { invalidate('stale'); return; }
      if (latest && time >= latest.expiresAt) invalidate('stale');
      const duration = previousReceipt === null ? 16 : Math.max(16, Math.min(330, time - previousReceipt));
      previousReceipt = time;
      cancelTimer();
      panel.accept?.(frame, time);
      if (!latest) baseline = frame;
      for (const [key, spec] of Object.entries(channels)) {
        const value = spec.get ? spec.get(frame) : frame[key];
        const valid = spec.valid(value, frame);
        const turnOverflow = key === 'turn_rate_dps' && finite(frame.turn_rate_dps) &&
          Math.abs(frame.turn_rate_dps) > DISPLAY_LIMIT;
        let state = states[key];
        if (!state) state = states[key] = { valid, value, target: value, baseValue: value, baseValid: valid, paintTime: time };
        else if (!valid) { state.valid = false; state.segment = state.pending = null; }
        else if (!state.valid) { Object.assign(state, { valid: true, value, target: value, segment: null, pending: null, snap: true, paintTime: time }); }
        else if (value !== state.target || (state.turnOverflow && !turnOverflow)) {
          state.target = value;
          state.pending = { value, time, duration };
        }
        // A recovery baseline that overflowed was never a painted edge anchor.
        if (baseline && turnOverflow) state.snap = true;
        state.turnOverflow = turnOverflow;
      }
      latest = frame;
      schedule();
    },
    resize(layout) { pendingLayout = layout; if (latest) schedule(); else applyLayout(); },
    redraw() { schedule(); }, invalidate,
  };
}

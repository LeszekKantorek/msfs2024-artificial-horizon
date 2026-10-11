import { svgElement, setAttributes, clamp } from '../helpers.js';
import { altitudeState, ALTITUDE_POINTER } from './readout.js';
import { tickPosition } from './tape.js';

const FLASH_DURATION = 5000;
const FLASH_PHASE = 250;
const BUG_HALF_HEIGHT = ALTITUDE_POINTER.halfHeight + 0.5;

export function initialAlert() {
  return { target: null, error: null, approached: false, captured: false,
    deviation: false, flashStarted: null, kind: null };
}

// Each accepted sample advances history. Rendering and resize never advance it.
export function advanceAlert(previous, altitude, target, now) {
  if (altitudeState(altitude) !== 'available' || altitudeState(target) !== 'available') return initialAlert();
  const error = Math.abs(altitude - target);
  if (previous.target !== target || previous.error === null) {
    return { ...initialAlert(), target, error, approached: error <= 1000, captured: error <= 200 };
  }
  const next = { ...previous, error };
  function flash(kind) { next.kind = kind; next.flashStarted = now; }
  if (error <= 200) {
    if (next.deviation) { next.deviation = false; next.flashStarted = null; next.kind = null; }
    if (!next.captured) { next.captured = true; next.approached = true; flash('within-200'); }
  } else if (next.captured && previous.error <= 200) {
    next.deviation = true; flash('deviation');
  } else if (!next.approached && previous.error > 1000 && error <= 1000) {
    next.approached = true; flash('within-1000');
  }
  return next;
}

export function alertPresentation(state, now) {
  const elapsed = state.flashStarted === null ? FLASH_DURATION : now - state.flashStarted;
  const flashing = elapsed >= 0 && elapsed < FLASH_DURATION;
  return { deviation: state.deviation, flashing,
    visible: !flashing || Math.floor(elapsed / FLASH_PHASE) % 2 === 0,
    nextUpdateAt: flashing ? state.flashStarted + Math.min(FLASH_DURATION,
      (Math.floor(elapsed / FLASH_PHASE) + 1) * FLASH_PHASE) : null };
}

export function selectedBugPosition(altitude, target, height) {
  return clamp(tickPosition(altitude, target, height), BUG_HALF_HEIGHT, height - BUG_HALF_HEIGHT);
}

export function createSelectedAltitude(parent, bugHost) {
  const { tip, shoulder, halfHeight } = ALTITUDE_POINTER;
  const symbol = `M 0 ${-halfHeight} H ${shoulder} L ${tip} 0 L ${shoulder} ${halfHeight} H 0 Z`;
  const group = svgElement('g', { id: 'selected-altitude', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'selected-altitude-window' });
  const icon = svgElement('path', { class: 'selected-altitude-symbol', d: symbol, transform: 'translate(3 16)' });
  const value = svgElement('text', { id: 'selected-altitude-value', class: 'selected-altitude-value' }, 'X');
  group.append(background, icon, value); parent.append(group);
  const bug = svgElement('path', { id: 'selected-altitude-bug', class: 'selected-altitude-bug', d: symbol });
  bugHost.append(bug);
  let geometry;
  let alert = initialAlert();
  return {
    resize(allocation) {
      geometry = allocation;
      group.setAttribute('transform', `translate(${allocation.x} 0)`);
      setAttributes(background, { x: 0.5, y: 0.5, width: allocation.width - 1, height: 31 });
      setAttributes(value, { x: allocation.width - 3, y: 16 });
    },
    accept(altitude, target, now) { alert = advanceAlert(alert, altitude, target, now); },
    invalidate() {
      alert = initialAlert();
      group.dataset.alert = 'none'; group.dataset.flashing = 'false';
      value.style.visibility = ''; bug.style.display = 'none';
    },
    render(altitude, target, now) {
      const state = altitudeState(target);
      const presentation = alertPresentation(alert, now);
      const rounded = Math.round(target);
      group.dataset.state = state;
      group.dataset.alert = presentation.deviation ? 'deviation' : presentation.flashing ? alert.kind : 'none';
      group.dataset.flashing = String(presentation.flashing);
      value.textContent = state === 'available' ? String(Object.is(rounded, -0) ? 0 : rounded) : state === 'overflow' ? 'OVR' : 'X';
      value.style.visibility = presentation.visible ? '' : 'hidden';
      group.setAttribute('aria-label', `Selected altitude ${state === 'available' ? `${target} feet` : state}${presentation.deviation ? ', deviation' : ''}`);
      const positioned = state === 'available' && altitudeState(altitude) === 'available';
      bug.style.display = positioned ? '' : 'none';
      if (positioned) bug.setAttribute('transform', `translate(0 ${selectedBugPosition(altitude, target, geometry.height)})`);
      return presentation.nextUpdateAt;
    },
  };
}

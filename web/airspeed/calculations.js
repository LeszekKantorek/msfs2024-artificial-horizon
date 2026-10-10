import { clamp } from '../helpers.js';

const PIXELS_PER_KNOT = 4;
export const MINOR_TICK_STEP_KT = 2;
export const MAJOR_TICK_STEP_KT = 10;
export const DIGIT_HEIGHT = 22;
export const MAX_SPEED = 999;

// Missing/invalid data and display overflow are different presentation states.
export function speedState(value) {
  if (!Number.isFinite(value) || value < 0) return 'unavailable';
  return value > MAX_SPEED ? 'overflow' : 'available';
}

export function tapeOffset(speed, tick) { return (speed - tick) * PIXELS_PER_KNOT; }

// Compact fallback text preserves the configured value in the accessible label.
export function referenceReadout(value) {
  if (value > MAX_SPEED) return 'OVR';
  if (value < 0.1) return '<0.1';
  return String(Math.round(value * 10) / 10);
}

export function placeReferenceLabels(ias, values, tape) {
  const { centerY: center, clipTopY: top, clipBottomY: bottom } = tape;
  const positions = values.map(value => {
    const position = center + tapeOffset(ias, value);
    return { position, labelPosition: position, visible: position >= top + 8 && position <= bottom - 8 };
  });
  const visible = positions.filter(p => p.visible).sort((a, b) => a.position - b.position);
  // Pack colliding labels, then move the packed set back inside the clip if needed.
  for (let i = 1; i < visible.length; i++)
    visible[i].labelPosition = Math.max(visible[i].position, visible[i - 1].labelPosition + 16);
  if (visible.length && visible.at(-1).labelPosition > bottom - 8) {
    visible.at(-1).labelPosition = bottom - 8;
    for (let i = visible.length - 2; i >= 0; i--)
      visible[i].labelPosition = Math.min(visible[i].labelPosition, visible[i + 1].labelPosition - 16);
  }
  return positions;
}

export function speedRanges(profile) {
  return [
    { name: 'low', min: 0, max: profile.vs0_kt },
    { name: 'flaps', min: profile.vs0_kt, max: profile.vfe_kt },
    { name: 'normal', min: profile.vs1_kt, max: profile.vno_kt },
    { name: 'caution', min: profile.vno_kt, max: profile.vne_kt },
    { name: 'overspeed', min: profile.vne_kt, max: Infinity },
  ];
}

export function speedIndications(ias, acceleration, profile) {
  const state = speedState(ias);
  const overspeed = state !== 'unavailable' && !!profile && ias >= profile.vne_kt;
  if (state !== 'available') return { prediction: null, overspeed, approaching: false };
  const predicted = Number.isFinite(acceleration) && acceleration !== 0 ? ias + 6 * acceleration : NaN;
  const prediction = Number.isFinite(predicted) ? predicted : null;
  return { prediction, overspeed,
    approaching: !!profile && prediction !== null && ias < profile.vne_kt && prediction >= profile.vne_kt };
}

// Higher columns roll only during the final knot before a decimal carry.
export function rollingDigits(value) {
  const whole = Math.floor(value);
  const fraction = value - whole;
  return [100, 10, 1].map(place => {
    const digit = Math.floor(whole / place) % 10;
    const rolling = place === 1 || whole % place === place - 1;
    const visible = place === 1 || whole >= place;
    return { current: visible ? String(digit) : '',
      next: whole + 1 >= place ? String((digit + 1) % 10) : '',
      offset: rolling ? fraction : 0 };
  });
}

export function calculateAirspeedGeometry(layout) {
  const { x, y, width, height } = layout.airspeed;
  const centerY = height / 2;
  const digitCenterX = (width - 14) / 2;
  const groundSpeedPanelY = layout.height - 24;
  return {
    tape: { originX: x, originY: y, width, centerY, clipTopY: 1, clipBottomY: height - 17,
      backgroundHeight: groundSpeedPanelY - y, unitY: height - 6,
      fallbackUnitY: groundSpeedPanelY - y - 80, fallbackFirstLabelY: groundSpeedPanelY - y - 64 },
    readout: { width, centerY, digitCenterX, warningPanelX: x + digitCenterX, warningPanelY: y + centerY },
    groundSpeed: { originX: x, originY: groundSpeedPanelY, width, valueX: width / 2 + 1,
      warningPanelX: x + width / 2 + 1, warningPanelY: groundSpeedPanelY + 11 },
  };
}

export function deriveSpeedReadings(frame) {
  const iasState = speedState(frame.ias_kt);
  const gsState = speedState(frame.gs_kt);
  const profile = frame.airspeed_profile;
  return {
    ias: { value: frame.ias_kt, state: iasState },
    groundSpeed: { state: gsState, roundedValue: gsState === 'available' ? Math.round(frame.gs_kt) : null },
    profile,
    ...speedIndications(frame.ias_kt, frame.ias_acceleration_ktps, profile),
    showBottomReferences: !!profile && iasState === 'unavailable',
  };
}

export function rangeBounds(iasKt, range, tape) {
  const top = clamp(tape.centerY + tapeOffset(iasKt, range.max), tape.clipTopY, tape.clipBottomY);
  const bottom = clamp(tape.centerY + tapeOffset(iasKt, range.min), tape.clipTopY, tape.clipBottomY);
  return { y: top, height: Math.max(0, bottom - top) };
}

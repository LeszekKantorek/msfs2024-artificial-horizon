import { createHorizon } from './horizon/index.js';
import { createFixedSymbols } from './fixed-symbols/index.js';
import { createSlipSkid } from './slip-skid.js';
import { createTurnRate } from './turn-rate.js';
import { createAirspeed } from './airspeed/index.js';
import { createAltitude } from './altitude/index.js';
import { createHeading } from './heading.js';
import { setAttributes } from './helpers.js';

// Delegation never selects instrument fields or computes instrument geometry.
export function composePanel(svg, instruments, status) {
  const attitudeClip = svg.querySelector('#attitude-clip rect');
  return {
    resize(layout) {
      svg.setAttribute('viewBox', `0 0 ${layout.width} ${layout.height}`);
      setAttributes(attitudeClip, layout.attitude);
      for (const instrument of instruments) instrument.resize(layout);
      status.resize?.(layout);
    },
    accept(frame, now) {
      for (const instrument of instruments) instrument.accept?.(frame, now);
    },
    render(frame, now) {
      let nextUpdateAt = null;
      for (const instrument of instruments) {
        const next = instrument.render?.(frame, now);
        if (Number.isFinite(next)) nextUpdateAt = nextUpdateAt === null ? next : Math.min(nextUpdateAt, next);
      }
      status.render(frame);
      return nextUpdateAt;
    },
    invalidate(reason) {
      status.invalidate(reason);
      for (const instrument of instruments) instrument.invalidate?.(reason);
    },
  };
}

export function createPanel(instrument, status, redraw = () => {}) {
  const layers = {
    background: instrument.querySelector('#layer-background'),
    airspeed: instrument.querySelector('#layer-airspeed'),
    altitude: instrument.querySelector('#layer-altitude'),
    symbols: instrument.querySelector('#layer-symbols'),
    warnings: instrument.querySelector('#layer-warnings'),
  };
  return composePanel(instrument.querySelector('svg'), [
    createHorizon(instrument, layers), createFixedSymbols(instrument, layers.symbols),
    createAirspeed(instrument, layers),
    createAltitude(instrument, layers, redraw),
    createHeading(instrument, layers),
    createSlipSkid(instrument, layers), createTurnRate(instrument, layers),
  ], status);
}

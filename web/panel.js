import { createHorizon } from './horizon.js';
import { createFixedSymbols } from './fixed-symbols.js';
import { createSlipSkid } from './slip-skid.js';
import { createTurnRate } from './turn-rate.js';

// Delegation never selects instrument fields or computes instrument geometry.
export function composePanel(svg, instruments, status) {
  return {
    resize(layout) {
      svg.setAttribute('viewBox', `0 0 ${layout.width} ${layout.height}`);
      for (const instrument of instruments) instrument.resize(layout);
    },
    render(frame) {
      for (const instrument of instruments) instrument.render?.(frame);
      status.render(frame);
    },
    invalidate(reason) { status.invalidate(reason); },
  };
}

export function createPanel(instrument, status) {
  return composePanel(instrument.querySelector('svg'), [
    createHorizon(instrument), createFixedSymbols(instrument),
    createSlipSkid(instrument), createTurnRate(instrument),
  ], status);
}

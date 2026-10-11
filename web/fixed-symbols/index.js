import { createBankScale } from './bank-scale.js';
import { createAircraft } from './aircraft.js';

export function createFixedSymbols(instrument, symbols) {
  const group = instrument.querySelector('#fixed-symbols');
  symbols.append(instrument.querySelector('#fixed-symbols-clip'));
  const bankScale = createBankScale(group.querySelector('#bank-scale'));
  const aircraft = createAircraft(group);
  return {
    resize(layout) {
      group.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
      bankScale.resize(layout.bankRadius);
      aircraft.resize(layout.aircraftWidth);
    },
  };
}

import { svgElement } from './svg.js';

const BANK_ANGLES = [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60];
const BANK_MAJOR_TICK = 12;
const BANK_MINOR_TICK = 7;
const AIRCRAFT_INNER_OFFSET = 25;
const AIRCRAFT_CHEVRON = 12;

export function createFixedSymbols(instrument, symbols) {
  const group = instrument.querySelector('#fixed-symbols');
  symbols.append(instrument.querySelector('#fixed-symbols-clip'));
  const bank = group.querySelector('#bank-scale');
  const zero = svgElement('path', { class: 'bank-zero' });
  bank.append(zero);
  const ticks = BANK_ANGLES.map(degrees => {
    const path = svgElement('path', {});
    bank.append(path);
    return { degrees, path };
  });
  const aircraft = group.querySelector('.aircraft');
  const outline = group.querySelector('.aircraft-outline');
  return {
    resize(layout) {
      const radius = layout.bankRadius;
      group.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
      zero.setAttribute('d', `M 0 ${-radius - 10} l -5 -8 h 10 Z`);
      for (const { degrees, path } of ticks) {
        path.setAttribute('d', `M 0 ${-radius} v ${degrees % 30 === 0 ? -BANK_MAJOR_TICK : -BANK_MINOR_TICK}`);
        path.setAttribute('transform', `rotate(${degrees} 0 0)`);
      }
      const half = layout.aircraftWidth / 2;
      const inner = AIRCRAFT_INNER_OFFSET, chevron = AIRCRAFT_CHEVRON;
      const shape = `M ${-half} 0 H ${-inner} l ${chevron} ${chevron} M ${inner - chevron} ${chevron} l ${chevron} -${chevron} H ${half} M -5 0 h 10`;
      aircraft.setAttribute('d', shape);
      outline.setAttribute('d', shape);
    },
  };
}

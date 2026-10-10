import { svgElement } from '../helpers.js';

const BANK_ANGLES = [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60];
const BANK_MAJOR_TICK = 12;
const BANK_MINOR_TICK = 7;

export function createBankScale(bank) {
  const zero = svgElement('path', { class: 'bank-zero' });
  bank.append(zero);
  const ticks = BANK_ANGLES.map(degrees => {
    const path = svgElement('path', {});
    bank.append(path);
    return { degrees, path };
  });
  return {
    resize(radius) {
      zero.setAttribute('d', `M 0 ${-radius - 10} l -5 -8 h 10 Z`);
      for (const { degrees, path } of ticks) {
        path.setAttribute('d', `M 0 ${-radius} v ${degrees % 30 === 0 ? -BANK_MAJOR_TICK : -BANK_MINOR_TICK}`);
        path.setAttribute('transform', `rotate(${degrees} 0 0)`);
      }
    },
  };
}

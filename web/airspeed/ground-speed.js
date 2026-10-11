import { svgElement, setAttributes } from '../helpers.js';
import { speedState } from './helpers.js';

export function createGroundSpeed(parent, warnings) {
  const group = svgElement('g', { id: 'ground-speed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background ground-speed-background' });
  const border = svgElement('rect', { class: 'ground-speed-border' });
  const label = svgElement('text', { class: 'ground-speed-label', y: 16 }, 'GS');
  const value = svgElement('text', { id: 'ground-speed-value', class: 'ground-speed-value', y: 16 });
  group.append(background, border, label, value); parent.append(group);
  const warning = svgElement('text', { id: 'ground-speed-warning', class: 'speed-warning' }, 'X');
  warnings.append(warning);
  return {
    resize({ x, y, width }) {
      const valueX = width - 3;
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: 32 });
      setAttributes(border, { x: 0.5, y: 0.5, width: width - 1, height: 31 });
      setAttributes(label, { x: 3 });
      setAttributes(value, { x: valueX });
      setAttributes(warning, { x: x + valueX, y: y + 16 });
    },
    render(speed) {
      const state = speedState(speed);
      const reading = { state, roundedValue: state === 'available' ? Math.round(speed) : null };
      group.dataset.state = reading.state;
      group.setAttribute('aria-label', `Ground speed ${reading.state === 'available' ? `${reading.roundedValue} knots` : reading.state}`);
      warning.style.display = reading.state === 'available' ? 'none' : '';
      warning.textContent = reading.state === 'overflow' ? 'OVR' : 'X';
      value.textContent = reading.state === 'available' ? String(reading.roundedValue) : '';
    },
  };
}

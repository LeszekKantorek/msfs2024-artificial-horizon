import { svgElement, setAttributes } from '../helpers.js';
import { speedState } from './helpers.js';

export function createTrueAirspeed(parent) {
  const group = svgElement('g', { id: 'true-airspeed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background true-airspeed-background' });
  const border = svgElement('rect', { class: 'true-airspeed-border' });
  const label = svgElement('text', { class: 'true-airspeed-label' }, 'TAS');
  const value = svgElement('text', { id: 'true-airspeed-value', class: 'true-airspeed-value' });
  group.append(background, border, label, value); parent.append(group);
  return {
    resize({ x, y, width, height }) {
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height });
      setAttributes(border, { x: 0.5, y: 0.5, width: width - 1, height: height - 1 });
      setAttributes(label, { x: 3, y: height / 2 });
      setAttributes(value, { x: width - 3, y: height / 2 });
    },
    render(speed) {
      const state = speedState(speed);
      const reading = state === 'available' ? String(Math.round(speed)) : state === 'overflow' ? 'OVR' : 'X';
      group.dataset.state = state;
      group.setAttribute('aria-label', `True airspeed ${state === 'available' ? `${reading} knots` : state}`);
      value.textContent = reading;
    },
  };
}

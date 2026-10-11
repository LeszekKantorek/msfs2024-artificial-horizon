import { svgElement, setAttributes } from '../helpers.js';

export function baroReading(value) {
  return typeof value === 'string' ? { state: 'available', text: value } : { state: 'unavailable', text: 'X' };
}

export function createBaro(parent) {
  const group = svgElement('g', { id: 'barometric-setting', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'barometric-background' });
  const border = svgElement('rect', { class: 'barometric-border' });
  const value = svgElement('text', { id: 'barometric-value', class: 'barometric-value' });
  group.append(background, border, value); parent.append(group);
  return {
    resize({ x, y, width }) {
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: 32 });
      setAttributes(border, { x: 0.5, y: 0.5, width: width - 1, height: 31 });
      setAttributes(value, { x: width / 2, y: 16 });
    },
    render(setting) {
      const reading = baroReading(setting);
      group.dataset.state = reading.state;
      value.textContent = reading.text;
      group.setAttribute('aria-label', `Barometric setting ${reading.state === 'available' ? reading.text : reading.state}`);
    },
  };
}

import { svgElement, setAttributes } from '../helpers.js';

export function baroReading(value) {
  if (!Number.isFinite(value) || value <= 0) return { state: 'unavailable', text: 'X' };
  if (value > 99.99) return { state: 'overflow', text: 'OVR' };
  return { state: 'available', text: value.toFixed(2) };
}

export function createBaro(parent) {
  const group = svgElement('g', { id: 'barometric-setting', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const value = svgElement('text', { id: 'barometric-value', class: 'barometric-value' });
  const unit = svgElement('text', { class: 'barometric-unit' }, 'inHg');
  group.append(background, value, unit); parent.append(group);
  return {
    resize({ x, y, width }) {
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: 32 });
      setAttributes(value, { x: width / 2, y: 10 });
      setAttributes(unit, { x: width / 2, y: 26 });
    },
    render(setting) {
      const reading = baroReading(setting);
      group.dataset.state = reading.state;
      value.textContent = reading.text;
      group.setAttribute('aria-label', `Barometric setting ${reading.state === 'available' ? `${setting} inches of mercury` : reading.state}`);
    },
  };
}

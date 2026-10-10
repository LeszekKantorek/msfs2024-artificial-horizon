import { svgElement, setAttributes } from '../helpers.js';

export function createGroundSpeed(parent, warnings) {
  const group = svgElement('g', { id: 'ground-speed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const label = svgElement('text', { class: 'ground-speed-label', y: 11 }, 'GS');
  const value = svgElement('text', { id: 'ground-speed-value', class: 'ground-speed-value', y: 11 });
  const unit = svgElement('text', { class: 'ground-speed-unit', y: 11 }, 'kt');
  group.append(background, label, value, unit); parent.append(group);
  const warning = svgElement('text', { id: 'ground-speed-warning', class: 'speed-warning' }, 'X');
  warnings.append(warning);
  return {
    resize(geometry) {
      group.setAttribute('transform', `translate(${geometry.originX} ${geometry.originY})`);
      setAttributes(background, { width: geometry.width, height: 22 });
      setAttributes(label, { x: 2 });
      setAttributes(value, { x: geometry.valueX });
      setAttributes(unit, { x: geometry.width - 2 });
      setAttributes(warning, { x: geometry.warningPanelX, y: geometry.warningPanelY });
    },
    render(reading) {
      group.dataset.state = reading.state;
      group.setAttribute('aria-label', `Ground speed ${reading.state === 'available' ? `${reading.roundedValue} knots` : reading.state}`);
      warning.style.display = reading.state === 'available' ? 'none' : '';
      warning.textContent = reading.state === 'overflow' ? 'OVR' : 'X';
      value.textContent = reading.state === 'available' ? String(reading.roundedValue) : '';
    },
  };
}

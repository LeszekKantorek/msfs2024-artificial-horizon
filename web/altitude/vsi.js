import { svgElement, setAttributes } from '../helpers.js';

export function verticalSpeedReading(fpm) {
  if (!Number.isFinite(fpm)) return { state: 'unavailable', text: 'X' };
  const rounded = Math.round(fpm);
  return {
    state: Math.abs(fpm) > 9999 ? 'overflow' : 'available',
    text: Math.abs(fpm) > 9999 ? `${fpm > 0 ? '+' : '-'}OVR` : rounded > 0 ? `+${rounded}` : rounded < 0 ? String(rounded) : '0',
  };
}

// Local altitude coordinates; include both strokes in the three-pixel gap.
export function vsiWindow({ width, height }) {
  return { x: width - 53, y: height / 2 + 18, width: 52, height: 28 };
}

export function createVsi(parent) {
  const group = svgElement('g', { id: 'vertical-speed', 'data-state': 'unavailable' });
  const window = svgElement('rect', { class: 'vertical-speed-window' });
  const value = svgElement('text', { id: 'vertical-speed-value', class: 'vertical-speed-value' }, 'X');
  group.append(window, value); parent.append(group);
  function invalidate() {
    group.dataset.state = 'unavailable';
    group.setAttribute('aria-label', 'Vertical speed unavailable');
    value.textContent = 'X';
  }
  invalidate();
  return {
    resize(allocation, textRight) {
      const geometry = vsiWindow(allocation);
      group.setAttribute('transform', `translate(${geometry.x} ${geometry.y})`);
      setAttributes(window, { width: geometry.width, height: geometry.height });
      setAttributes(value, { x: textRight - geometry.x, y: geometry.height / 2 });
    },
    invalidate,
    render(fpm) {
      const reading = verticalSpeedReading(fpm);
      if (reading.state === 'unavailable') { invalidate(); return; }
      group.dataset.state = reading.state;
      value.textContent = reading.text;
      group.setAttribute('aria-label', `Vertical speed ${fpm} feet per minute`);
    },
  };
}

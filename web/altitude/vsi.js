import { svgElement, setAttributes } from '../helpers.js';

export function verticalSpeedReading(fpm) {
  if (!Number.isFinite(fpm)) return { state: 'unavailable', text: 'X' };
  const magnitude = Math.round(Math.abs(fpm) / 10) * 10;
  const rounded = fpm < 0 ? -magnitude : magnitude;
  return {
    state: Math.abs(fpm) > 9999 ? 'overflow' : 'available',
    text: Math.abs(fpm) > 9999 ? `${fpm > 0 ? '+' : '-'}OVR` : rounded > 0 ? `+${rounded}` : rounded < 0 ? String(rounded) : '0',
  };
}

// Local altitude coordinates; include both strokes in the three-pixel gap.
export function vsiWindow({ width, height }) {
  return { x: width - 52, y: height / 2 + 18, width: 51, height: 27 };
}

export function createVsi(parent) {
  const group = svgElement('g', { id: 'vertical-speed', 'data-state': 'unavailable' });
  const window = svgElement('rect', { class: 'vertical-speed-window' });
  const value = svgElement('text', { id: 'vertical-speed-value', class: 'vertical-speed-value' }, 'X');
  group.append(window, value); parent.append(group);
  let textRight;
  function alignValue() {
    if (textRight === undefined) return;
    value.setAttribute('x', textRight);
    // Glyph bounds can extend beyond the end anchor.
    const bounds = value.getBBox();
    value.setAttribute('x', textRight - (bounds.x + bounds.width - textRight));
  }
  function setValue(text) {
    if (value.textContent === text) return;
    value.textContent = text;
    alignValue();
  }
  function invalidate() {
    group.dataset.state = 'unavailable';
    group.setAttribute('aria-label', 'Vertical speed unavailable');
    setValue('X');
  }
  invalidate();
  return {
    resize(allocation, altitudeTextRight) {
      const geometry = vsiWindow(allocation);
      group.setAttribute('transform', `translate(${geometry.x} ${geometry.y})`);
      setAttributes(window, { width: geometry.width, height: geometry.height });
      textRight = altitudeTextRight - geometry.x;
      setAttributes(value, { y: geometry.height / 2 });
      alignValue();
    },
    invalidate,
    render(fpm) {
      const reading = verticalSpeedReading(fpm);
      if (reading.state === 'unavailable') { invalidate(); return; }
      group.dataset.state = reading.state;
      setValue(reading.text);
      group.setAttribute('aria-label', `Vertical speed ${fpm} feet per minute`);
    },
  };
}

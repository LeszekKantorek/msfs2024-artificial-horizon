import { svgElement, setAttributes, clamp } from './helpers.js';

export function verticalSpeedReading(fpm) {
  if (!Number.isFinite(fpm)) return { state: 'unavailable', text: 'X', offScale: false };
  const rounded = Math.round(fpm);
  return {
    state: Math.abs(fpm) > 9999 ? 'overflow' : 'available',
    text: Math.abs(fpm) > 9999 ? `${fpm > 0 ? '+' : '-'}OVR` : rounded > 0 ? `+${rounded}` : rounded < 0 ? String(rounded) : '0',
    offScale: Math.abs(fpm) > 2000,
  };
}

export function verticalSpeedPosition(fpm, height) {
  return height / 2 - clamp(fpm, -2000, 2000) / 2000 * (height / 2 - 8);
}

export function createVerticalSpeed(instrument, layers) {
  const group = svgElement('g', { id: 'vertical-speed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const scale = svgElement('g', { id: 'vertical-speed-scale' });
  const clip = svgElement('clipPath', { id: 'vertical-speed-clip' });
  const clipRect = svgElement('rect', {});
  clip.append(clipRect); instrument.querySelector('defs').append(clip);
  const pointerHost = svgElement('g', { 'clip-path': 'url(#vertical-speed-clip)' });
  const pointer = svgElement('g', { id: 'vertical-speed-pointer' });
  const arrow = svgElement('path', { class: 'vertical-speed-arrow', d: 'M 2 0 L 9 -5 V 5 Z' });
  const overflow = svgElement('path', { id: 'vertical-speed-off-scale', class: 'vertical-speed-off-scale' });
  pointer.append(arrow, overflow); pointerHost.append(pointer);
  group.append(background, scale, pointerHost); layers.altitude.append(group);
  const unit = svgElement('text', { class: 'vertical-speed-unit' }, 'FT/MIN');
  const window = svgElement('rect', { class: 'vertical-speed-window' });
  const value = svgElement('text', { id: 'vertical-speed-value', class: 'vertical-speed-value' }, 'X');
  group.append(unit, window, value);
  const ticks = [];
  for (let fpm = -2000; fpm <= 2000; fpm += 500) {
    const line = svgElement('path', { class: 'vertical-speed-tick' });
    const label = fpm % 1000 === 0 ? svgElement('text', { class: 'vertical-speed-label' }, String(Math.abs(fpm))) : null;
    scale.append(line); if (label) scale.append(label);
    ticks.push({ fpm, line, label });
  }
  let geometry;
  function invalidate() {
    group.dataset.state = 'unavailable'; group.dataset.offScale = 'false';
    group.setAttribute('aria-label', 'Vertical speed unavailable');
    value.textContent = 'X'; pointer.style.display = 'none'; overflow.style.display = 'none';
  }
  invalidate();
  return {
    resize(layout) {
      geometry = layout.vsi;
      const { x, y, width, height } = geometry;
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: layout.height - 34 - y });
      setAttributes(clipRect, { width, height });
      setAttributes(unit, { x: width / 2, y: -16 });
      setAttributes(window, { x: 0.5, y: layout.height - 34 - y + 0.5, width: width - 1, height: 31 });
      setAttributes(value, { x: width / 2, y: layout.height - 18 - y });
      for (const tick of ticks) {
        const pos = verticalSpeedPosition(tick.fpm, height);
        tick.line.setAttribute('d', `M ${width - (tick.label ? 6 : 4)} ${pos} H ${width - 1}`);
        if (tick.label) setAttributes(tick.label, { x: width - 8, y: pos });
      }
    },
    invalidate,
    render(frame) {
      const fpm = frame.vertical_speed_fpm;
      const reading = verticalSpeedReading(fpm);
      if (reading.state === 'unavailable') { invalidate(); return null; }
      group.dataset.state = reading.state; group.dataset.offScale = String(reading.offScale);
      value.textContent = reading.text;
      group.setAttribute('aria-label', `Vertical speed ${fpm} feet per minute${reading.offScale ? ', off scale' : ''}`);
      pointer.style.display = '';
      pointer.setAttribute('transform', `translate(0 ${verticalSpeedPosition(fpm, geometry.height)})`);
      overflow.style.display = reading.offScale ? '' : 'none';
      overflow.setAttribute('d', fpm > 0 ? 'M 2 -3 L 5 -6 L 8 -3' : 'M 2 3 L 5 6 L 8 3');
      return null;
    },
  };
}

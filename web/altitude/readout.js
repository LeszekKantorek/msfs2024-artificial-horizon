import { svgElement, setAttributes } from '../helpers.js';

const DIGIT_HEIGHT = 20;
const MIN_ALTITUDE = -9999;
const MAX_ALTITUDE = 99999;

export function altitudeState(value) {
  if (!Number.isFinite(value)) return 'unavailable';
  return value < MIN_ALTITUDE || value > MAX_ALTITUDE ? 'overflow' : 'available';
}

// The last two digits form one 20-foot drum. Higher columns carry over its final row.
export function rollingAltitude(value) {
  const magnitude = Math.abs(value);
  const base = Math.floor(magnitude / 20) * 20;
  const fraction = (magnitude - base) / 20;
  const nextValue = base + 20;
  const limit = value < 0 ? -MIN_ALTITUDE : MAX_ALTITUDE;
  const columns = [10000, 1000, 100].map(place => ({
    current: magnitude >= place ? String(Math.floor(base / place) % 10) : '',
    next: nextValue <= limit && nextValue >= place ? String(Math.floor(nextValue / place) % 10) : '',
    offset: base % place >= place - 20 ? fraction : 0,
  }));
  columns.push({ current: String(base % 100).padStart(2, '0'),
    next: nextValue <= limit ? String(nextValue % 100).padStart(2, '0') : '', offset: fraction });
  return { sign: value < 0 ? '−' : '', columns };
}

export function createReadout(defs, parent, warnings) {
  const clip = svgElement('clipPath', { id: 'altitude-digits-clip' });
  const rect = svgElement('rect', {});
  clip.append(rect); defs.append(clip);
  const window = svgElement('path', { class: 'altitude-window' });
  const digits = svgElement('g', { id: 'altitude-digits', 'clip-path': 'url(#altitude-digits-clip)' });
  const sign = svgElement('text', { id: 'altitude-sign', class: 'altitude-digit' });
  digits.append(sign);
  const columns = Array.from({ length: 4 }, (_, index) => {
    const group = svgElement('g', {});
    const current = svgElement('text', { class: 'altitude-digit', y: 0 }, index === 3 ? '00' : '');
    const next = svgElement('text', { class: 'altitude-digit', y: -DIGIT_HEIGHT });
    group.append(current, next); digits.append(group);
    return { group, current, next };
  });
  parent.append(window, digits);
  const warning = svgElement('text', { id: 'altitude-warning', class: 'altitude-warning' }, 'X');
  warnings.append(warning);
  let geometry;
  return {
    resize(readout) {
      geometry = readout;
      const { width, height, x, y } = readout;
      const center = height / 2;
      window.setAttribute('d', `M 1 ${center} L 7 ${center - 14} H ${width - 1} V ${center + 14} H 7 Z`);
      setAttributes(rect, { x: 7, y: center - DIGIT_HEIGHT / 2, width: width - 9, height: DIGIT_HEIGHT });
      setAttributes(sign, { x: 11, y: center });
      setAttributes(warning, { x: x + width / 2, y: y + center });
      // Two monospace digits keep the same width during rolling and unavailable states.
      return width - 10.5 + columns[3].current.getComputedTextLength() / 2;
    },
    render(value, state) {
      warning.style.display = state === 'available' ? 'none' : '';
      warning.textContent = state === 'overflow' ? 'OVR' : 'X';
      if (state !== 'available') { sign.textContent = ''; return; }
      const reading = rollingAltitude(value);
      sign.textContent = reading.sign;
      reading.columns.forEach((digit, index) => {
        const { group, current, next } = columns[index];
        const offsetX = [41.5, 32.5, 23.5, 10.5][index];
        group.setAttribute('transform', `translate(${geometry.width - offsetX} ${geometry.height / 2 + digit.offset * DIGIT_HEIGHT})`);
        current.textContent = digit.current; next.textContent = digit.next;
      });
    },
  };
}

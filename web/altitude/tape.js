import { svgElement, setAttributes } from '../helpers.js';

const MINOR_STEP = 20;
const MAJOR_STEP = 100;
const SPAN = 400;
const TICK_COUNT = SPAN / MINOR_STEP + 3;

export function tickPosition(altitude, tick, height) {
  return height / 2 + (altitude - tick) * height / SPAN;
}

export function altitudeTicks(altitude, height) {
  const first = Math.floor((altitude - SPAN / 2) / MINOR_STEP) * MINOR_STEP - MINOR_STEP;
  return Array.from({ length: TICK_COUNT }, (_, index) => {
    const value = first + index * MINOR_STEP;
    return { value, y: tickPosition(altitude, value, height), major: value % MAJOR_STEP === 0 };
  });
}

export function createTape(parent) {
  const scale = svgElement('g', { id: 'altitude-scale' });
  const rows = Array.from({ length: TICK_COUNT }, () => {
    const tick = svgElement('path', { class: 'altitude-tick' });
    const label = svgElement('text', { class: 'altitude-label' });
    scale.append(tick, label);
    return { tick, label };
  });
  parent.append(scale);
  let geometry;
  return {
    resize(tape) { geometry = tape; },
    render(value) {
      if (value === null) return;
      altitudeTicks(value, geometry.height).forEach((row, index) => {
        const { tick, label } = rows[index];
        setAttributes(tick, { d: `M 1 ${row.y} H ${row.major ? 12 : 7}`, 'data-altitude-ft': row.value });
        setAttributes(label, { x: geometry.width - 3, y: row.y });
        label.textContent = row.major ? String(row.value) : '';
      });
    },
  };
}

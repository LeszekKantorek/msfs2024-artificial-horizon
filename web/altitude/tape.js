import { svgElement, setAttributes } from '../helpers.js';

const MINOR_STEP = 20;
const MAJOR_STEP = 100;
// Match the 160-pixel tape in the minimum-height landscape layout.
const MINOR_SPACING_PX = 8;
const PIXELS_PER_FOOT = MINOR_SPACING_PX / MINOR_STEP;

function tickCount(height) {
  return Math.ceil(height / MINOR_SPACING_PX) + 3;
}

export function tickPosition(altitude, tick, height) {
  return height / 2 + (altitude - tick) * PIXELS_PER_FOOT;
}

export function altitudeTicks(altitude, height) {
  const halfSpan = height / (2 * PIXELS_PER_FOOT);
  const first = Math.floor((altitude - halfSpan) / MINOR_STEP) * MINOR_STEP - MINOR_STEP;
  return Array.from({ length: tickCount(height) }, (_, index) => {
    const value = first + index * MINOR_STEP;
    return { value, y: tickPosition(altitude, value, height), major: value % MAJOR_STEP === 0 };
  });
}

export function createTape(parent) {
  const scale = svgElement('g', { id: 'altitude-scale' });
  const rows = [];
  parent.append(scale);
  let geometry;
  return {
    resize(tape) {
      geometry = tape;
      const count = tickCount(tape.height);
      // Grow only on resize and retain existing nodes through rotation and recovery.
      while (rows.length < count) {
        const tick = svgElement('path', { class: 'altitude-tick' });
        const label = svgElement('text', { class: 'altitude-label' });
        scale.append(tick, label);
        rows.push({ tick, label });
      }
      rows.forEach(({ tick, label }, index) => {
        tick.style.display = label.style.display = index < count ? '' : 'none';
      });
    },
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

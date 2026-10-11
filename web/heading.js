import { svgElement, setAttributes, clamp } from './helpers.js';

const PIXELS_PER_DEGREE = 4;
const WINDOW = Object.freeze({ width: 44, height: 22, pointerHalfWidth: 8, pointerHeight: 8 });
const BUG = Object.freeze({ halfWidth: 8, top: 24, height: 7, stroke: 1 });
const LABEL_HALF_WIDTH = 11;

export function validHeading(value) {
  return Number.isFinite(value) && value >= 0 && value < 360;
}

export function headingText(value) {
  if (!validHeading(value)) return 'X';
  return String(Math.round(value) || 360).padStart(3, '0');
}

export function headingDifference(heading, selected) {
  return (selected - heading + 540) % 360 - 180;
}

export function headingBugPosition(heading, selected, width) {
  if (!validHeading(heading) || !validHeading(selected)) return null;
  const inset = BUG.halfWidth + BUG.stroke / 2;
  return clamp(width / 2 + headingDifference(heading, selected) * PIXELS_PER_DEGREE, inset, width - inset);
}

export function headingTicks(heading, width) {
  if (!validHeading(heading)) return [];
  const first = Math.floor(heading - width / (2 * PIXELS_PER_DEGREE)) - 1;
  return Array.from({ length: Math.ceil(width / PIXELS_PER_DEGREE) + 3 }, (_, index) => {
    const degrees = first + index;
    const value = (degrees % 360 + 360) % 360;
    const x = width / 2 + (degrees - heading) * PIXELS_PER_DEGREE;
    const major = value % 5 === 0;
    const labelVisible = value % 10 === 0 && x - LABEL_HALF_WIDTH >= 0 && x + LABEL_HALF_WIDTH <= width &&
      Math.abs(x - width / 2) > WINDOW.width / 2 + 1 + LABEL_HALF_WIDTH;
    return { value, x, major, labelVisible };
  });
}

export function createHeading(instrument, layers) {
  const defs = instrument.querySelector('defs');
  const group = svgElement('g', { id: 'heading', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const clip = svgElement('clipPath', { id: 'heading-tape-clip' });
  const rect = svgElement('rect', {});
  clip.append(rect); defs.append(clip);
  const tape = svgElement('g', { id: 'heading-scale' });
  const bug = svgElement('path', { id: 'selected-heading-bug', class: 'selected-heading-bug' });
  const window = svgElement('path', { class: 'heading-window' });
  const value = svgElement('text', { id: 'heading-value', class: 'heading-value' }, 'X');
  group.setAttribute('clip-path', 'url(#heading-tape-clip)');
  group.append(background, tape, bug, window, value); layers.symbols.append(group);
  const rows = [];
  let geometry;
  function invalidate() {
    group.dataset.state = 'unavailable'; group.dataset.selectedAvailable = 'false';
    group.setAttribute('aria-label', 'Magnetic heading unavailable');
    tape.style.display = bug.style.display = 'none'; value.textContent = 'X';
    bug.removeAttribute('transform');
  }
  invalidate();
  return {
    resize(layout) {
      geometry = layout.heading;
      const { x, y, width, height } = geometry;
      const center = width / 2, shoulder = WINDOW.pointerHalfWidth;
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(rect, { x: 0, y: 0, width, height });
      setAttributes(background, { width, height });
      window.setAttribute('d', `M ${center - WINDOW.width / 2} 0.5 H ${center + WINDOW.width / 2} V ${WINDOW.height} H ${center + shoulder} L ${center} ${WINDOW.height + WINDOW.pointerHeight} L ${center - shoulder} ${WINDOW.height} H ${center - WINDOW.width / 2} Z`);
      setAttributes(value, { x: center, y: 11 });
      const notchHalfWidth = shoulder * (WINDOW.pointerHeight - (BUG.top - WINDOW.height)) / WINDOW.pointerHeight;
      bug.setAttribute('d', `M ${-BUG.halfWidth} ${BUG.top} H ${-notchHalfWidth} L 0 ${WINDOW.height + WINDOW.pointerHeight} L ${notchHalfWidth} ${BUG.top} H ${BUG.halfWidth} V ${BUG.top + BUG.height} H ${-BUG.halfWidth} Z`);
      const count = Math.ceil(width / PIXELS_PER_DEGREE) + 3;
      while (rows.length < count) {
        const tick = svgElement('path', { class: 'heading-tick' });
        const label = svgElement('text', { class: 'heading-label', y: 13 });
        tape.append(tick, label); rows.push({ tick, label });
      }
      rows.forEach(({ tick, label }, index) => { tick.style.display = label.style.display = index < count ? '' : 'none'; });
    },
    invalidate,
    render(frame) {
      const heading = frame.heading_deg, selected = frame.selected_heading_deg;
      if (!validHeading(heading)) { invalidate(); return; }
      group.dataset.state = 'available';
      group.dataset.selectedAvailable = String(validHeading(selected));
      group.setAttribute('aria-label', `Magnetic heading ${heading} degrees${validHeading(selected) ? `, selected heading ${selected} degrees` : ', selected heading unavailable'}`);
      tape.style.display = '';
      value.textContent = headingText(heading);
      headingTicks(heading, geometry.width).forEach((row, index) => {
        const { tick, label } = rows[index];
        setAttributes(tick, { d: `M ${row.x} ${row.major ? 23 : 27} V 31`, 'data-heading-deg': row.value });
        label.setAttribute('x', row.x);
        label.textContent = row.value % 10 === 0 ? headingText(row.value) : '';
        label.style.visibility = row.labelVisible ? '' : 'hidden';
      });
      const position = headingBugPosition(heading, selected, geometry.width);
      bug.style.display = position === null ? 'none' : '';
      if (position === null) bug.removeAttribute('transform');
      else bug.setAttribute('transform', `translate(${position} 0)`);
    },
  };
}

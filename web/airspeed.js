import { svgElement, setAttributes } from './svg.js';

const PIXELS_PER_KNOT = 4;
const DIGIT_HEIGHT = 22;
const MAX_SPEED = 999;

// Missing/invalid data and display overflow are different presentation states.
export function speedState(value) {
  if (!Number.isFinite(value) || value < 0) return 'unavailable';
  return value > MAX_SPEED ? 'overflow' : 'available';
}

export function tapeOffset(speed, tick) { return (speed - tick) * PIXELS_PER_KNOT; }

// Higher columns roll only during the final knot before a decimal carry.
export function rollingDigits(value) {
  const whole = Math.floor(value);
  const fraction = value - whole;
  return [100, 10, 1].map(place => {
    const digit = Math.floor(whole / place) % 10;
    const rolling = place === 1 || whole % place === place - 1;
    const visible = place === 1 || whole >= place;
    return { current: visible ? String(digit) : '',
      next: whole + 1 >= place ? String((digit + 1) % 10) : '',
      offset: rolling ? fraction : 0 };
  });
}

export function createAirspeed(instrument, layers) {
  const defs = instrument.querySelector('defs');
  const tapeClip = svgElement('clipPath', { id: 'airspeed-tape-clip' });
  const tapeRect = svgElement('rect', {});
  tapeClip.append(tapeRect); defs.append(tapeClip);
  const digitClip = svgElement('clipPath', { id: 'airspeed-digits-clip' });
  const digitRect = svgElement('rect', {});
  digitClip.append(digitRect); defs.append(digitClip);
  const ias = svgElement('g', { id: 'airspeed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const tape = svgElement('g', { class: 'airspeed-tape', 'clip-path': 'url(#airspeed-tape-clip)' });
  const scale = svgElement('g', { id: 'airspeed-scale' });
  const ticks = svgElement('path', { class: 'airspeed-ticks' });
  const labels = Array.from({ length: 100 }, (_, index) =>
    svgElement('text', { class: 'airspeed-label', y: -index * 40 }, String(index * 10)));
  scale.append(ticks, ...labels); tape.append(scale);
  const window = svgElement('path', { class: 'airspeed-window' });
  const digits = svgElement('g', { id: 'airspeed-digits', 'clip-path': 'url(#airspeed-digits-clip)' });
  const columns = Array.from({ length: 3 }, () => {
    const column = svgElement('g', {});
    const current = svgElement('text', { class: 'airspeed-digit', y: 0 });
    const next = svgElement('text', { class: 'airspeed-digit', y: -DIGIT_HEIGHT });
    column.append(current, next); digits.append(column);
    return { column, current, next };
  });
  const unit = svgElement('text', { class: 'speed-unit' }, 'KT');
  ias.append(background, tape, window, digits, unit);
  layers.airspeed.append(ias);
  const iasWarning = svgElement('text', { id: 'airspeed-warning', class: 'speed-warning' }, 'X');
  layers.warnings.append(iasWarning);
  const gs = svgElement('g', { id: 'ground-speed', 'data-state': 'unavailable' });
  const gsBackground = svgElement('rect', { class: 'instrument-background' });
  const gsLabel = svgElement('text', { class: 'ground-speed-label', y: 11 }, 'GS');
  const gsValue = svgElement('text', { id: 'ground-speed-value', class: 'ground-speed-value', y: 11 });
  const gsUnit = svgElement('text', { class: 'ground-speed-unit', y: 11 }, 'kt');
  gs.append(gsBackground, gsLabel, gsValue, gsUnit); layers.airspeed.append(gs);
  const gsWarning = svgElement('text', { id: 'ground-speed-warning', class: 'speed-warning' }, 'X');
  layers.warnings.append(gsWarning);
  let center, digitX;
  return {
    resize(layout) {
      const { x, y, width, height } = layout.airspeed;
      center = height / 2;
      digitX = (width - 6) / 2;
      ias.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height });
      setAttributes(tapeRect, { x: 0, y: 1, width, height: height - 18 });
      ticks.setAttribute('d', Array.from({ length: 500 }, (_, index) => {
        const major = index % 5 === 0;
        return `M ${width - (major ? 10 : 6)} ${-index * 8} H ${width - 2}`;
      }).join(' '));
      for (const label of labels) label.setAttribute('x', width - 13);
      window.setAttribute('d', `M 1 ${center - 14} H ${width - 7} L ${width - 1} ${center} L ${width - 7} ${center + 14} H 1 Z`);
      setAttributes(digitRect, { x: 2, y: center - 11, width: width - 10, height: 22 });
      setAttributes(unit, { x: width / 2, y: height - 6 });
      setAttributes(iasWarning, { x: x + digitX, y: y + center });
      const gsY = y + height;
      gs.setAttribute('transform', `translate(${x} ${gsY})`);
      setAttributes(gsBackground, { width, height: 22 });
      setAttributes(gsLabel, { x: 2 });
      setAttributes(gsValue, { x: width / 2 + 1 });
      setAttributes(gsUnit, { x: width - 2 });
      setAttributes(gsWarning, { x: x + width / 2 + 1, y: gsY + 11 });
    },
    render(frame) {
      const iasState = speedState(frame.ias_kt);
      const gsState = speedState(frame.gs_kt);
      ias.dataset.state = iasState;
      gs.dataset.state = gsState;
      ias.setAttribute('aria-label', `Indicated airspeed ${iasState === 'available' ? `${frame.ias_kt} knots` : iasState}`);
      gs.setAttribute('aria-label', `Ground speed ${gsState === 'available' ? `${Math.round(frame.gs_kt)} knots` : gsState}`);
      iasWarning.style.display = iasState === 'available' ? 'none' : '';
      gsWarning.style.display = gsState === 'available' ? 'none' : '';
      iasWarning.textContent = iasState === 'overflow' ? 'OVR' : 'X';
      gsWarning.textContent = gsState === 'overflow' ? 'OVR' : 'X';
      if (iasState === 'available') {
        scale.setAttribute('transform', `translate(0 ${center + tapeOffset(frame.ias_kt, 0)})`);
        rollingDigits(frame.ias_kt).forEach((digit, index) => {
          const { column, current, next } = columns[index];
          column.setAttribute('transform', `translate(${digitX + (index - 1) * 10} ${center + digit.offset * DIGIT_HEIGHT})`);
          current.textContent = digit.current; next.textContent = digit.next;
        });
      }
      gsValue.textContent = gsState === 'available' ? String(Math.round(frame.gs_kt)) : '';
    },
  };
}

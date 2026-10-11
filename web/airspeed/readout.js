import { svgElement, setAttributes } from '../helpers.js';
const DIGIT_HEIGHT = 22;

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

export function createReadout(defs, parent, warnings) {
  const digitClip = svgElement('clipPath', { id: 'airspeed-digits-clip' });
  const digitRect = svgElement('rect', {});
  digitClip.append(digitRect); defs.append(digitClip);
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
  parent.append(window, digits, unit);
  const warning = svgElement('text', { id: 'airspeed-warning', class: 'speed-warning' }, 'X');
  warnings.append(warning);
  let geometry;
  return {
    resize(readout, unitPosition) {
      geometry = readout;
      const { width, centerY } = readout;
      window.setAttribute('d', `M 1 ${centerY - 14} H ${width - 16} L ${width - 11} ${centerY} L ${width - 16} ${centerY + 14} H 1 Z`);
      setAttributes(digitRect, { x: 2, y: centerY - DIGIT_HEIGHT / 2, width: width - 18, height: DIGIT_HEIGHT });
      setAttributes(unit, { x: unitPosition.unitX, y: unitPosition.unitY });
      setAttributes(warning, { x: readout.warningPanelX, y: readout.warningPanelY });
    },
    render(ias, showBottomReferences) {
      unit.style.display = showBottomReferences ? 'none' : '';
      warning.style.display = ias.state === 'available' ? 'none' : '';
      warning.textContent = ias.state === 'overflow' ? 'OVR' : 'X';
      if (ias.state !== 'available') return;
      rollingDigits(ias.value).forEach((digit, index) => {
        const { column, current, next } = columns[index];
        column.setAttribute('transform', `translate(${geometry.digitCenterX + (index - 1) * 10} ${geometry.centerY + digit.offset * DIGIT_HEIGHT})`);
        current.textContent = digit.current; next.textContent = digit.next;
      });
    },
  };
}

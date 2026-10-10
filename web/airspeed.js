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

// Compact fallback text preserves the configured value in the accessible label.
export function referenceReadout(value) {
  if (value > MAX_SPEED) return 'OVR';
  if (value < 0.1) return '<0.1';
  return String(Math.round(value * 10) / 10);
}

export function referenceGeometry(ias, values, center, top, bottom) {
  const positions = values.map(value => {
    const position = center + tapeOffset(ias, value);
    return { position, labelPosition: position, visible: position >= top + 8 && position <= bottom - 8 };
  });
  const visible = positions.filter(p => p.visible).sort((a, b) => a.position - b.position);
  // Pack colliding labels, then move the packed set back inside the clip if needed.
  for (let i = 1; i < visible.length; i++)
    visible[i].labelPosition = Math.max(visible[i].position, visible[i - 1].labelPosition + 16);
  if (visible.length && visible.at(-1).labelPosition > bottom - 8) {
    visible.at(-1).labelPosition = bottom - 8;
    for (let i = visible.length - 2; i >= 0; i--)
      visible[i].labelPosition = Math.min(visible[i].labelPosition, visible[i + 1].labelPosition - 16);
  }
  return positions;
}

export function speedRanges(profile) {
  return [
    { name: 'low', min: 0, max: profile.vs0_kt },
    { name: 'flaps', min: profile.vs0_kt, max: profile.vfe_kt },
    { name: 'normal', min: profile.vs1_kt, max: profile.vno_kt },
    { name: 'caution', min: profile.vno_kt, max: profile.vne_kt },
    { name: 'overspeed', min: profile.vne_kt, max: Infinity },
  ];
}

export function speedIndications(ias, acceleration, profile) {
  const state = speedState(ias);
  const overspeed = state !== 'unavailable' && !!profile && ias >= profile.vne_kt;
  if (state !== 'available') return { prediction: null, overspeed, approaching: false };
  const predicted = Number.isFinite(acceleration) && acceleration !== 0 ? ias + 6 * acceleration : NaN;
  const prediction = Number.isFinite(predicted) ? predicted : null;
  return { prediction, overspeed,
    approaching: !!profile && prediction !== null && ias < profile.vne_kt && prediction >= profile.vne_kt };
}

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
  const barberPole = svgElement('pattern', { id: 'airspeed-barber-pole', patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(30)', width: 6, height: 12 });
  barberPole.append(svgElement('rect', { width: 6, height: 12, class: 'airspeed-range-white' }),
    svgElement('path', { d: 'M 0 0 H 6 V 6 H 0 Z', class: 'airspeed-range-red' }));
  defs.append(barberPole);
  const ias = svgElement('g', { id: 'airspeed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const tape = svgElement('g', { class: 'airspeed-tape', 'clip-path': 'url(#airspeed-tape-clip)' });
  const scale = svgElement('g', { id: 'airspeed-scale' });
  const ticks = svgElement('path', { class: 'airspeed-ticks' });
  const labels = Array.from({ length: 100 }, (_, index) =>
    svgElement('text', { class: 'airspeed-label', y: -index * 40 }, String(index * 10)));
  scale.append(ticks, ...labels);
  const ranges = svgElement('g', { id: 'airspeed-ranges' });
  const rangeRects = ['low', 'flaps', 'normal', 'caution', 'overspeed'].map(name => {
    const rect = svgElement('rect', { 'data-range': name, class: `airspeed-range-${name}` });
    ranges.append(rect); return rect;
  });
  const trend = svgElement('path', { id: 'airspeed-trend', class: 'airspeed-trend' });
  const references = svgElement('g', { id: 'airspeed-references' });
  const referenceKeys = ['vr_kt', 'vx_kt', 'vy_kt', 'vg_kt'];
  const referenceNames = ['R', 'X', 'Y', 'G'];
  const leaders = referenceNames.map(name => {
    const path = svgElement('path', { class: 'airspeed-reference-leader', 'data-reference': name });
    references.append(path); return path;
  });
  const markers = referenceNames.map(name => {
    const text = svgElement('text', { 'data-reference': name, class: 'airspeed-reference' }, name);
    references.append(text); return text;
  });
  tape.append(scale, ranges, trend, references);
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
  const bottomReferences = svgElement('g', { id: 'airspeed-references-bottom', 'data-visible': 'false' });
  const bottomUnit = svgElement('text', { class: 'speed-unit' }, 'KT');
  const bottomLabels = referenceNames.map(name => svgElement('text', { class: 'airspeed-reference-value', 'data-reference': name }));
  bottomReferences.append(bottomUnit, ...bottomLabels);
  ias.append(background, tape, window, digits, unit, bottomReferences);
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
  let center, digitX, tapeTop, tapeBottom, tapeWidth;
  return {
    resize(layout) {
      const { x, y, width, height } = layout.airspeed;
      center = height / 2;
      digitX = (width - 14) / 2;
      tapeWidth = width;
      tapeTop = 1;
      tapeBottom = height - 17;
      const gsY = layout.height - 24;
      ias.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: gsY - y });
      setAttributes(tapeRect, { x: 0, y: 1, width, height: height - 18 });
      ticks.setAttribute('d', Array.from({ length: 500 }, (_, index) => {
        const major = index % 5 === 0;
        return `M ${width - (major ? 26 : 22)} ${-index * 8} H ${width - 18}`;
      }).join(' '));
      for (const label of labels) label.setAttribute('x', width - 28);
      window.setAttribute('d', `M 1 ${center - 14} H ${width - 16} L ${width - 11} ${center} L ${width - 16} ${center + 14} H 1 Z`);
      setAttributes(digitRect, { x: 2, y: center - 11, width: width - 18, height: 22 });
      markers.forEach(marker => marker.setAttribute('x', width - 4));
      setAttributes(bottomUnit, { x: width / 2, y: gsY - y - 80 });
      bottomLabels.forEach((label, i) => setAttributes(label, { x: width / 2, y: gsY - y - 64 + i * 16 }));
      setAttributes(unit, { x: width / 2, y: height - 6 });
      setAttributes(iasWarning, { x: x + digitX, y: y + center });
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
      const profile = frame.airspeed_profile;
      const { prediction, overspeed, approaching } = speedIndications(frame.ias_kt, frame.ias_acceleration_ktps, profile);
      ias.dataset.overspeed = String(overspeed);
      ias.dataset.approaching = String(approaching);
      ranges.style.display = profile && iasState === 'available' ? '' : 'none';
      references.style.display = profile && iasState === 'available' ? '' : 'none';
      const showBottom = !!profile && iasState === 'unavailable';
      bottomReferences.dataset.visible = String(showBottom);
      bottomReferences.style.display = showBottom ? '' : 'none';
      unit.style.display = showBottom ? 'none' : '';
      if (profile) {
        const values = referenceKeys.map(key => profile.references[key]);
        const positions = referenceGeometry(frame.ias_kt, values, center, tapeTop, tapeBottom);
        referenceKeys.forEach((key, i) => {
          const value = profile.references[key];
          bottomLabels[i].textContent = `${referenceNames[i]} ${referenceReadout(value)}`;
          const { position, labelPosition, visible } = positions[i];
          markers[i].style.display = iasState === 'available' && visible ? '' : 'none';
          markers[i].setAttribute('y', Number.isFinite(labelPosition) ? labelPosition : 0);
          const displaced = iasState === 'available' && visible && labelPosition !== position;
          leaders[i].setAttribute('d', displaced ? `M ${tapeWidth - 7} ${position} V ${labelPosition} H ${tapeWidth - 5}` : '');
          markers[i].setAttribute('aria-label', `${referenceKeys[i].slice(0, -3).toUpperCase()} ${value} knots`);
        });
        bottomReferences.setAttribute('aria-label', `Configured reference speeds in knots: VR ${profile.references.vr_kt}, VX ${profile.references.vx_kt}, VY ${profile.references.vy_kt}, VG ${profile.references.vg_kt}`);
        if (iasState === 'available') speedRanges(profile).forEach((range, i) => {
          const top = Math.max(tapeTop, Math.min(tapeBottom, center + tapeOffset(frame.ias_kt, range.max)));
          const bottom = Math.max(tapeTop, Math.min(tapeBottom, center + tapeOffset(frame.ias_kt, range.min)));
          setAttributes(rangeRects[i], { x: tapeWidth - (range.name === 'flaps' ? 17 : 15),
            width: range.name === 'flaps' ? 2 : 3, y: top, height: Math.max(0, bottom - top) });
        });
      }
      trend.style.display = prediction === null ? 'none' : '';
      trend.setAttribute('d', prediction === null ? '' : `M ${tapeWidth - 10} ${center} V ${Math.max(tapeTop + 1, Math.min(tapeBottom - 1, center + tapeOffset(frame.ias_kt, prediction)))}`);
      if (prediction === null) trend.removeAttribute('data-prediction-kt');
      else trend.setAttribute('data-prediction-kt', prediction);
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

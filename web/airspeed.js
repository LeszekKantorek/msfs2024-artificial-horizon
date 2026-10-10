import { clamp, svgElement, setAttributes } from './helpers.js';
import { MINOR_TICK_STEP_KT, MAJOR_TICK_STEP_KT, DIGIT_HEIGHT, MAX_SPEED,
  tapeOffset, referenceReadout, placeReferenceLabels, speedRanges, rollingDigits,
  calculateAirspeedGeometry, deriveSpeedReadings, rangeBounds } from './airspeed/calculations.js';

function createAirspeedElements(instrument, layers) {
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
  const labels = Array.from({ length: Math.floor(MAX_SPEED / MAJOR_TICK_STEP_KT) + 1 }, (_, index) => {
    const speed = index * MAJOR_TICK_STEP_KT;
    return svgElement('text', { class: 'airspeed-label', y: tapeOffset(0, speed) }, String(speed));
  });
  scale.append(ticks, ...labels);
  const ranges = svgElement('g', { id: 'airspeed-ranges' });
  const rangeRects = Object.fromEntries(['low', 'flaps', 'normal', 'caution', 'overspeed'].map(name => {
    const rect = svgElement('rect', { 'data-range': name, class: `airspeed-range-${name}` });
    ranges.append(rect); return [name, rect];
  }));
  const trend = svgElement('path', { id: 'airspeed-trend', class: 'airspeed-trend' });
  const references = svgElement('g', { id: 'airspeed-references' });
  const referenceRecords = [
    { key: 'vr_kt', symbol: 'R', accessibleName: 'VR' },
    { key: 'vx_kt', symbol: 'X', accessibleName: 'VX' },
    { key: 'vy_kt', symbol: 'Y', accessibleName: 'VY' },
    { key: 'vg_kt', symbol: 'G', accessibleName: 'VG' },
  ].map(reference => ({ ...reference,
    leader: svgElement('path', { class: 'airspeed-reference-leader', 'data-reference': reference.symbol }),
    marker: svgElement('text', { 'data-reference': reference.symbol, class: 'airspeed-reference' }, reference.symbol),
    fallbackLabel: svgElement('text', { class: 'airspeed-reference-value', 'data-reference': reference.symbol }),
  }));
  references.append(...referenceRecords.map(reference => reference.leader), ...referenceRecords.map(reference => reference.marker));
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
  bottomReferences.append(bottomUnit, ...referenceRecords.map(reference => reference.fallbackLabel));
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
  return {
    ias, background,
    tape: { clipRect: tapeRect, scale, ticks, labels, ranges: { group: ranges, rects: rangeRects }, trend,
      references: { group: references, records: referenceRecords, bottomGroup: bottomReferences, bottomUnit } },
    readout: { window, clipRect: digitRect, columns, unit, warning: iasWarning },
    groundSpeed: { group: gs, background: gsBackground, label: gsLabel, value: gsValue, unit: gsUnit, warning: gsWarning },
  };
}

function resizeAirspeedElements(elements, geometry) {
  const { tape, readout, groundSpeed } = geometry;
  elements.ias.setAttribute('transform', `translate(${tape.originX} ${tape.originY})`);
  setAttributes(elements.background, { width: tape.width, height: tape.backgroundHeight });
  setAttributes(elements.tape.clipRect, { x: 0, y: tape.clipTopY, width: tape.width, height: tape.clipBottomY - tape.clipTopY });
  elements.tape.ticks.setAttribute('d', Array.from({ length: Math.floor(MAX_SPEED / MINOR_TICK_STEP_KT) + 1 }, (_, index) => {
    const speed = index * MINOR_TICK_STEP_KT;
    const major = speed % MAJOR_TICK_STEP_KT === 0;
    return `M ${tape.width - (major ? 26 : 22)} ${tapeOffset(0, speed)} H ${tape.width - 18}`;
  }).join(' '));
  for (const label of elements.tape.labels) label.setAttribute('x', tape.width - 28);
  const { width, centerY } = readout;
  elements.readout.window.setAttribute('d', `M 1 ${centerY - 14} H ${width - 16} L ${width - 11} ${centerY} L ${width - 16} ${centerY + 14} H 1 Z`);
  setAttributes(elements.readout.clipRect, { x: 2, y: centerY - DIGIT_HEIGHT / 2, width: width - 18, height: DIGIT_HEIGHT });
  const references = elements.tape.references;
  setAttributes(references.bottomUnit, { x: tape.width / 2, y: tape.fallbackUnitY });
  references.records.forEach((reference, index) => {
    reference.marker.setAttribute('x', tape.width - 4);
    setAttributes(reference.fallbackLabel, { x: tape.width / 2, y: tape.fallbackFirstLabelY + index * 16 });
  });
  setAttributes(elements.readout.unit, { x: tape.width / 2, y: tape.unitY });
  setAttributes(elements.readout.warning, { x: readout.warningPanelX, y: readout.warningPanelY });
  elements.groundSpeed.group.setAttribute('transform', `translate(${groundSpeed.originX} ${groundSpeed.originY})`);
  setAttributes(elements.groundSpeed.background, { width: groundSpeed.width, height: 22 });
  setAttributes(elements.groundSpeed.label, { x: 2 });
  setAttributes(elements.groundSpeed.value, { x: groundSpeed.valueX });
  setAttributes(elements.groundSpeed.unit, { x: groundSpeed.width - 2 });
  setAttributes(elements.groundSpeed.warning, { x: groundSpeed.warningPanelX, y: groundSpeed.warningPanelY });
}

function updateSpeedStates(elements, readings) {
  const { ias, groundSpeed, profile, overspeed, approaching, showBottomReferences } = readings;
  elements.ias.dataset.overspeed = String(overspeed);
  elements.ias.dataset.approaching = String(approaching);
  elements.tape.ranges.group.style.display = profile && ias.state === 'available' ? '' : 'none';
  elements.tape.references.group.style.display = profile && ias.state === 'available' ? '' : 'none';
  elements.tape.references.bottomGroup.dataset.visible = String(showBottomReferences);
  elements.tape.references.bottomGroup.style.display = showBottomReferences ? '' : 'none';
  elements.readout.unit.style.display = showBottomReferences ? 'none' : '';
  elements.ias.dataset.state = ias.state;
  elements.groundSpeed.group.dataset.state = groundSpeed.state;
  elements.ias.setAttribute('aria-label', `Indicated airspeed ${ias.state === 'available' ? `${ias.value} knots` : ias.state}`);
  elements.groundSpeed.group.setAttribute('aria-label', `Ground speed ${groundSpeed.state === 'available' ? `${groundSpeed.roundedValue} knots` : groundSpeed.state}`);
  elements.readout.warning.style.display = ias.state === 'available' ? 'none' : '';
  elements.groundSpeed.warning.style.display = groundSpeed.state === 'available' ? 'none' : '';
  elements.readout.warning.textContent = ias.state === 'overflow' ? 'OVR' : 'X';
  elements.groundSpeed.warning.textContent = groundSpeed.state === 'overflow' ? 'OVR' : 'X';
}

function renderTape(scale, tape, ias) {
  if (ias.state === 'available') scale.setAttribute('transform', `translate(0 ${tape.centerY + tapeOffset(ias.value, 0)})`);
}

function renderRanges(rects, tape, ias, profile) {
  if (!profile || ias.state !== 'available') return;
  for (const range of speedRanges(profile)) {
    setAttributes(rects[range.name], { x: tape.width - (range.name === 'flaps' ? 17 : 15),
      width: range.name === 'flaps' ? 2 : 3, ...rangeBounds(ias.value, range, tape) });
  }
}

function renderTrend(trend, tape, ias, prediction) {
  trend.style.display = prediction === null ? 'none' : '';
  trend.setAttribute('d', prediction === null ? '' : `M ${tape.width - 10} ${tape.centerY} V ${clamp(tape.centerY + tapeOffset(ias.value, prediction), tape.clipTopY + 1, tape.clipBottomY - 1)}`);
  if (prediction === null) trend.removeAttribute('data-prediction-kt');
  else trend.setAttribute('data-prediction-kt', prediction);
}

function renderReferences(references, tape, ias, profile) {
  if (!profile) return;
  const values = references.records.map(reference => profile.references[reference.key]);
  const positions = placeReferenceLabels(ias.value, values, tape);
  references.records.forEach((reference, index) => {
    const value = values[index];
    const { marker, leader, fallbackLabel, symbol, accessibleName } = reference;
    fallbackLabel.textContent = `${symbol} ${referenceReadout(value)}`;
    const { position, labelPosition, visible } = positions[index];
    marker.style.display = ias.state === 'available' && visible ? '' : 'none';
    marker.setAttribute('y', Number.isFinite(labelPosition) ? labelPosition : 0);
    const displaced = ias.state === 'available' && visible && labelPosition !== position;
    leader.setAttribute('d', displaced ? `M ${tape.width - 7} ${position} V ${labelPosition} H ${tape.width - 5}` : '');
    marker.setAttribute('aria-label', `${accessibleName} ${value} knots`);
  });
  references.bottomGroup.setAttribute('aria-label', `Configured reference speeds in knots: ${references.records.map(reference => `${reference.accessibleName} ${profile.references[reference.key]}`).join(', ')}`);
}

function renderDigits(columns, readout, ias) {
  if (ias.state !== 'available') return;
  rollingDigits(ias.value).forEach((digit, index) => {
    const { column, current, next } = columns[index];
    column.setAttribute('transform', `translate(${readout.digitCenterX + (index - 1) * 10} ${readout.centerY + digit.offset * DIGIT_HEIGHT})`);
    current.textContent = digit.current; next.textContent = digit.next;
  });
}

function renderGroundSpeed(value, reading) {
  value.textContent = reading.state === 'available' ? String(reading.roundedValue) : '';
}

export function createAirspeed(instrument, layers) {
  const elements = createAirspeedElements(instrument, layers);
  let geometry;
  return {
    resize(layout) {
      geometry = calculateAirspeedGeometry(layout);
      resizeAirspeedElements(elements, geometry);
    },
    render(frame) {
      const readings = deriveSpeedReadings(frame);
      updateSpeedStates(elements, readings);
      renderTape(elements.tape.scale, geometry.tape, readings.ias);
      renderRanges(elements.tape.ranges.rects, geometry.tape, readings.ias, readings.profile);
      renderTrend(elements.tape.trend, geometry.tape, readings.ias, readings.prediction);
      renderReferences(elements.tape.references, geometry.tape, readings.ias, readings.profile);
      renderDigits(elements.readout.columns, geometry.readout, readings.ias);
      renderGroundSpeed(elements.groundSpeed.value, readings.groundSpeed);
    },
  };
}

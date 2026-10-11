import { clamp, svgElement, setAttributes } from '../helpers.js';
import { MAX_SPEED, tapeOffset } from './helpers.js';

const MINOR_TICK_STEP_KT = 2;
const MAJOR_TICK_STEP_KT = 10;

export function speedRanges(profile) {
  return [
    { name: 'low', min: 0, max: profile.vs0_kt },
    { name: 'flaps', min: profile.vs0_kt, max: profile.vfe_kt },
    { name: 'normal', min: profile.vs1_kt, max: profile.vno_kt },
    { name: 'caution', min: profile.vno_kt, max: profile.vne_kt },
    { name: 'overspeed', min: profile.vne_kt, max: Infinity },
  ];
}

export function rangeBounds(iasKt, range, tape) {
  const top = clamp(tape.centerY + tapeOffset(iasKt, range.max), tape.clipTopY, tape.clipBottomY);
  const bottom = clamp(tape.centerY + tapeOffset(iasKt, range.min), tape.clipTopY, tape.clipBottomY);
  return { y: top, height: Math.max(0, bottom - top) };
}

export function createTape(defs, parent) {
  const barberPole = svgElement('pattern', { id: 'airspeed-barber-pole', patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(30)', width: 6, height: 12 });
  barberPole.append(svgElement('rect', { width: 6, height: 12, class: 'airspeed-range-white' }),
    svgElement('path', { d: 'M 0 0 H 6 V 6 H 0 Z', class: 'airspeed-range-red' }));
  defs.append(barberPole);
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
  parent.append(scale, ranges, trend);
  let geometry;
  function renderRanges(ias, profile) {
    ranges.style.display = profile && ias.state === 'available' ? '' : 'none';
    if (!profile || ias.state !== 'available') return;
    for (const range of speedRanges(profile)) {
      setAttributes(rangeRects[range.name], { x: geometry.width - (range.name === 'flaps' ? 17 : 15),
        width: range.name === 'flaps' ? 2 : 3, ...rangeBounds(ias.value, range, geometry) });
    }
  }
  function renderTrend(ias, prediction) {
    trend.style.display = prediction === null ? 'none' : '';
    trend.setAttribute('d', prediction === null ? '' : `M ${geometry.width - 10} ${geometry.centerY} V ${clamp(geometry.centerY + tapeOffset(ias.value, prediction), geometry.clipTopY + 1, geometry.clipBottomY - 1)}`);
    if (prediction === null) trend.removeAttribute('data-prediction-kt');
    else trend.setAttribute('data-prediction-kt', prediction);
  }
  return {
    resize(tape) {
      geometry = tape;
      ticks.setAttribute('d', Array.from({ length: Math.floor(MAX_SPEED / MINOR_TICK_STEP_KT) + 1 }, (_, index) => {
        const speed = index * MINOR_TICK_STEP_KT;
        const major = speed % MAJOR_TICK_STEP_KT === 0;
        return `M ${tape.width - (major ? 26 : 22)} ${tapeOffset(0, speed)} H ${tape.width - 18}`;
      }).join(' '));
      for (const label of labels) label.setAttribute('x', tape.width - 28);
    },
    render(ias, profile, prediction) {
      if (ias.state === 'available') scale.setAttribute('transform', `translate(0 ${geometry.centerY + tapeOffset(ias.value, 0)})`);
      renderRanges(ias, profile);
      renderTrend(ias, prediction);
    },
  };
}

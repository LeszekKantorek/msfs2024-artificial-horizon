import { svgElement, setAttributes, clamp } from '../helpers.js';
import { altitudeState } from './readout.js';
import { tickPosition } from './tape.js';

// Source ft/min predicts six seconds; resize keeps the tape's fixed pixel scale.
export function altitudeTrend(altitude, fpm, height) {
  if (altitudeState(altitude) !== 'available' || !Number.isFinite(fpm) || fpm === 0) return null;
  const change = fpm / 10;
  return { prediction: altitude + change, y: clamp(tickPosition(0, change, height), 1, height - 1) };
}

export function createAltitudeTrend(defs, parent) {
  const clip = svgElement('clipPath', { id: 'altitude-trend-clip' });
  const rect = svgElement('rect', {});
  clip.append(rect); defs.append(clip);
  const line = svgElement('path', { id: 'altitude-trend', class: 'altitude-trend',
    'clip-path': 'url(#altitude-trend-clip)' });
  parent.append(line);
  let geometry;
  function invalidate() {
    line.style.display = 'none';
    line.removeAttribute('d'); line.removeAttribute('data-prediction-ft');
  }
  invalidate();
  return {
    resize(lane) {
      geometry = lane;
      setAttributes(rect, { x: lane.x, y: 0, width: lane.width, height: lane.height });
    },
    invalidate,
    render(altitude, fpm) {
      const trend = altitudeTrend(altitude, fpm, geometry.height);
      if (!trend) { invalidate(); return; }
      const x = geometry.x + geometry.width / 2;
      line.style.display = '';
      setAttributes(line, { d: `M ${x} ${geometry.height / 2} V ${trend.y}`,
        'data-prediction-ft': trend.prediction });
    },
  };
}

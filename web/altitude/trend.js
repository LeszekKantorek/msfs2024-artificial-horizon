import { svgElement, setAttributes, clamp } from '../helpers.js';
import { altitudeState } from './readout.js';
import { tickPosition } from './tape.js';

// Source ft/min predicts six seconds; resize keeps the tape's fixed pixel scale.
export function altitudePrediction(altitude, fpm) {
  if (altitudeState(altitude) !== 'available' || !Number.isFinite(fpm) || fpm === 0) return null;
  const prediction = altitude + fpm / 10;
  return Number.isFinite(prediction) ? prediction : null;
}
function trendY(fpm, height) {
  return clamp(tickPosition(0, fpm / 10, height), 1, height - 1);
}
export function altitudeTrend(altitude, fpm, height) {
  const prediction = altitudePrediction(altitude, fpm);
  return prediction === null ? null : { prediction, y: trendY(fpm, height) };
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
    render(altitude, fpm, rawAltitude = altitude, rawFpm = fpm) {
      const prediction = altitudePrediction(rawAltitude, rawFpm);
      if (prediction === null) { invalidate(); return; }
      const y = trendY(fpm, geometry.height);
      const x = geometry.x + geometry.width / 2;
      line.style.display = '';
      setAttributes(line, { d: `M ${x} ${geometry.height / 2} V ${y}`,
        'data-prediction-ft': prediction });
    },
  };
}

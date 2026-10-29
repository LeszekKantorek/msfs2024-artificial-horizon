import { svgElement, setAttributes } from '../helpers.js';
import { createTape } from './tape.js';
import { createReadout, altitudeState } from './readout.js';
import { createBaro } from './baro.js';
import { createSelectedAltitude } from './selected.js';
import { createVsi, vsiWindow } from './vsi.js';
import { createAltitudeTrend } from './trend.js';

export function createAltitude(instrument, layers, redraw = () => {}) {
  const defs = instrument.querySelector('defs');
  const group = svgElement('g', { id: 'altitude', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const clip = svgElement('clipPath', { id: 'altitude-tape-clip' });
  const rect = svgElement('rect', {});
  clip.append(rect); defs.append(clip);
  const tapeHost = svgElement('g', { class: 'altitude-tape', 'clip-path': 'url(#altitude-tape-clip)' });
  group.append(background, tapeHost); layers.altitude.append(group);
  const tape = createTape(tapeHost);
  const trend = createAltitudeTrend(defs, group);
  const bugHost = svgElement('g', { class: 'selected-altitude-tape', 'clip-path': 'url(#altitude-tape-clip)' });
  group.append(bugHost);
  const readout = createReadout(defs, group, layers.warnings);
  const vsi = createVsi(group);
  const unit = svgElement('text', { class: 'altitude-unit' }, '[FT]');
  group.append(unit);
  const baro = createBaro(layers.altitude, redraw);
  const selected = createSelectedAltitude(layers.altitude, bugHost);
  return {
    resize(layout) {
      const allocation = layout.altitude;
      const width = allocation.width;
      group.setAttribute('transform', `translate(${allocation.x} ${allocation.y})`);
      setAttributes(background, { width, height: layout.height - 32 - allocation.y });
      setAttributes(rect, { x: 0, y: 0, width: allocation.width, height: allocation.height });
      setAttributes(unit, { x: allocation.width / 2, y: allocation.height + 8 });
      tape.resize(allocation, vsiWindow(allocation));
      const textRight = readout.resize(allocation);
      vsi.resize(allocation, textRight);
      trend.resize({ x: 0, width: 4, height: allocation.height });
      selected.resize(allocation);
      baro.resize({ x: allocation.x, y: layout.height - 32, width });
    },
    accept(frame, now) { selected.accept(frame.altitude_ft, frame.selected_altitude_ft, now); },
    invalidate() { selected.invalidate(); vsi.invalidate(); trend.invalidate(); baro.invalidate(); },
    render(frame, now) {
      const state = altitudeState(frame.altitude_ft);
      group.dataset.state = state;
      group.setAttribute('aria-label', `Barometric altitude ${state === 'available' ? `${frame.altitude_ft} feet` : state}`);
      tape.render(state === 'available' ? frame.altitude_ft : null);
      readout.render(frame.altitude_ft, state);
      baro.render(frame.baro_text, frame.baro_setting, Object.hasOwn(frame, 'baro_setting'));
      vsi.render(frame.vertical_speed_fpm);
      trend.render(frame.altitude_ft, frame.vertical_speed_fpm);
      return selected.render(frame.altitude_ft, frame.selected_altitude_ft, now);
    },
  };
}

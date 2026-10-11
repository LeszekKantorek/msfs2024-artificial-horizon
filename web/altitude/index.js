import { svgElement, setAttributes } from '../helpers.js';
import { createTape } from './tape.js';
import { createReadout, altitudeState } from './readout.js';
import { createBaro } from './baro.js';
import { createSelectedAltitude } from './selected.js';

export function createAltitude(instrument, layers) {
  const defs = instrument.querySelector('defs');
  const group = svgElement('g', { id: 'altitude', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const clip = svgElement('clipPath', { id: 'altitude-tape-clip' });
  const rect = svgElement('rect', {});
  clip.append(rect); defs.append(clip);
  const tapeHost = svgElement('g', { class: 'altitude-tape', 'clip-path': 'url(#altitude-tape-clip)' });
  group.append(background, tapeHost); layers.altitude.append(group);
  const tape = createTape(tapeHost);
  const readout = createReadout(defs, group, layers.warnings);
  const unit = svgElement('text', { class: 'altitude-unit' }, 'FT');
  group.append(unit);
  const bugHost = svgElement('g', { class: 'selected-altitude-tape', 'clip-path': 'url(#altitude-tape-clip)' });
  group.append(bugHost);
  const baro = createBaro(layers.altitude);
  const selected = createSelectedAltitude(layers.altitude, bugHost);
  return {
    resize(layout) {
      const allocation = layout.altitude;
      group.setAttribute('transform', `translate(${allocation.x} ${allocation.y})`);
      setAttributes(background, { width: allocation.width, height: layout.height - 34 - allocation.y });
      setAttributes(rect, { x: 0, y: 0, width: allocation.width, height: allocation.height });
      setAttributes(unit, { x: allocation.width / 2, y: allocation.height + 9 });
      tape.resize(allocation); readout.resize(allocation);
      selected.resize(allocation);
      baro.resize({ x: allocation.x, y: layout.height - 34, width: allocation.width });
    },
    accept(frame, now) { selected.accept(frame.altitude_ft, frame.selected_altitude_ft, now); },
    invalidate() { selected.invalidate(); },
    render(frame, now) {
      const state = altitudeState(frame.altitude_ft);
      group.dataset.state = state;
      group.setAttribute('aria-label', `Barometric altitude ${state === 'available' ? `${frame.altitude_ft} feet` : state}`);
      tape.render(state === 'available' ? frame.altitude_ft : null);
      readout.render(frame.altitude_ft, state);
      baro.render(frame.baro_inhg);
      return selected.render(frame.altitude_ft, frame.selected_altitude_ft, now);
    },
  };
}

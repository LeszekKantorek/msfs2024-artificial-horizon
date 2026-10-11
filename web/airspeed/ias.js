import { svgElement, setAttributes } from '../helpers.js';
import { speedState } from './helpers.js';
import { createTape } from './tape.js';
import { createReadout } from './readout.js';
import { createReferences } from './references.js';

export function speedIndications(ias, acceleration, profile) {
  const state = speedState(ias);
  const overspeed = state !== 'unavailable' && !!profile && ias >= profile.vne_kt;
  if (state !== 'available') return { prediction: null, overspeed, approaching: false };
  const predicted = Number.isFinite(acceleration) && acceleration !== 0 ? ias + 6 * acceleration : NaN;
  const prediction = Number.isFinite(predicted) ? predicted : null;
  return { prediction, overspeed,
    approaching: !!profile && prediction !== null && ias < profile.vne_kt && prediction >= profile.vne_kt };
}

function deriveIasReadings(value, profile, acceleration) {
  const state = speedState(value);
  return {
    ias: { value, state }, profile,
    ...speedIndications(value, acceleration, profile),
    showBottomReferences: !!profile && state === 'unavailable',
  };
}

export function createIas(defs, parent, warnings) {
  const tapeClip = svgElement('clipPath', { id: 'airspeed-tape-clip' });
  const tapeRect = svgElement('rect', {});
  tapeClip.append(tapeRect); defs.append(tapeClip);
  const group = svgElement('g', { id: 'airspeed', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'instrument-background' });
  const tapeHost = svgElement('g', { class: 'airspeed-tape', 'clip-path': 'url(#airspeed-tape-clip)' });
  group.append(background, tapeHost);
  // Digit clipping precedes the tape pattern in defs; the tape host already fixes paint order.
  const readout = createReadout(defs, group, warnings);
  const tape = createTape(defs, tapeHost);
  const references = createReferences(tapeHost, group);
  parent.append(group);
  return {
    resize({ x, y, width, height }, groundSpeedPanelY) {
      const centerY = height / 2;
      const digitCenterX = (width - 14) / 2;
      const tapeGeometry = { originX: x, originY: y, width, centerY, clipTopY: 0, clipBottomY: height,
        backgroundHeight: groundSpeedPanelY - y, unitY: height + 9,
        fallbackFirstLabelY: groundSpeedPanelY - y - 72 };
      const readoutGeometry = { width, centerY, digitCenterX,
        warningPanelX: x + digitCenterX, warningPanelY: y + centerY };
      group.setAttribute('transform', `translate(${tapeGeometry.originX} ${tapeGeometry.originY})`);
      setAttributes(background, { width: tapeGeometry.width, height: tapeGeometry.backgroundHeight });
      setAttributes(tapeRect, { x: 0, y: tapeGeometry.clipTopY, width: tapeGeometry.width,
        height: tapeGeometry.clipBottomY - tapeGeometry.clipTopY });
      tape.resize(tapeGeometry);
      readout.resize(readoutGeometry, { unitX: tapeGeometry.width / 2, unitY: tapeGeometry.unitY });
      references.resize(tapeGeometry);
    },
    render(value, profile, acceleration) {
      const { ias, prediction, overspeed, approaching, showBottomReferences } = deriveIasReadings(value, profile, acceleration);
      group.dataset.overspeed = String(overspeed);
      group.dataset.approaching = String(approaching);
      group.dataset.state = ias.state;
      group.setAttribute('aria-label', `Indicated airspeed ${ias.state === 'available' ? `${ias.value} knots` : ias.state}`);
      tape.render(ias, profile, prediction);
      readout.render(ias);
      references.render(ias, profile, showBottomReferences);
    },
  };
}

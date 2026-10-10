import { svgElement, setAttributes } from '../helpers.js';
import { createTape } from './tape.js';
import { createReadout } from './readout.js';
import { createReferences } from './references.js';

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
    resize(tapeGeometry, readoutGeometry) {
      group.setAttribute('transform', `translate(${tapeGeometry.originX} ${tapeGeometry.originY})`);
      setAttributes(background, { width: tapeGeometry.width, height: tapeGeometry.backgroundHeight });
      setAttributes(tapeRect, { x: 0, y: tapeGeometry.clipTopY, width: tapeGeometry.width,
        height: tapeGeometry.clipBottomY - tapeGeometry.clipTopY });
      tape.resize(tapeGeometry);
      readout.resize(readoutGeometry, { unitX: tapeGeometry.width / 2, unitY: tapeGeometry.unitY });
      references.resize(tapeGeometry);
    },
    render({ ias, profile, prediction, overspeed, approaching, showBottomReferences }) {
      group.dataset.overspeed = String(overspeed);
      group.dataset.approaching = String(approaching);
      group.dataset.state = ias.state;
      group.setAttribute('aria-label', `Indicated airspeed ${ias.state === 'available' ? `${ias.value} knots` : ias.state}`);
      tape.render(ias, profile, prediction);
      readout.render(ias, showBottomReferences);
      references.render(ias, profile, showBottomReferences);
    },
  };
}

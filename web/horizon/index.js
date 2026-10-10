import { setAttributes } from '../helpers.js';
import { attitudeTransforms } from './calculations.js';
import { createWorld } from './world.js';
import { createPitchScale } from './pitch-scale.js';
import { createBankPointer } from './bank-pointer.js';

export function createHorizon(instrument, layers) {
  const get = selector => instrument.querySelector(selector);
  const worldOrigin = get('.moving-world');
  const warningOrigin = get('#warning-origin');
  const backgroundClip = get('#background-clip rect');
  const pitchClip = get('#pitch-clip rect');
  layers.background.append(worldOrigin);
  layers.symbols.append(get('#bank-pointer-clip'));
  layers.warnings.append(warningOrigin);
  const world = createWorld(get('#world-rotation'));
  const pitchScale = createPitchScale(get('#scale-rotation'), get('#warning-rotation'));
  const bankPointer = createBankPointer(get('#bank-pointer-origin'));
  let localLayout;

  function resize(layout) {
    localLayout = { cx: 0, cy: 0, pitchScale: layout.pitchScale };
    const { attitude: attitudeRegion, bankRadius: radius } = layout;
    worldOrigin.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
    warningOrigin.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
    setAttributes(backgroundClip, layout.background);
    const pitchClipClearance = 25;
    setAttributes(pitchClip, { x: attitudeRegion.x - layout.cx, y: -radius + pitchClipClearance,
      width: attitudeRegion.width, height: attitudeRegion.y + attitudeRegion.height - (layout.cy - radius + pitchClipClearance) });
    world.resize({ width: layout.width, height: layout.height, pitchScale: layout.pitchScale });
    pitchScale.resize({ pitchScale: layout.pitchScale, labelSize: layout.labelSize });
    bankPointer.resize({ cx: layout.cx, cy: layout.cy, bankRadius: layout.bankRadius });
  }

  function render(frame) {
    const transforms = attitudeTransforms(frame.attitude, localLayout);
    world.render(transforms);
    pitchScale.render(transforms);
    bankPointer.render(transforms);
  }
  return { resize, render };
}

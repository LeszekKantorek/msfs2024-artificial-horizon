import { svgElement, setAttributes as attributes } from './svg.js';

const PITCH_MAJOR_HALF = 30;
const PITCH_MEDIUM_HALF = 16;
const PITCH_MINOR_HALF = 8;
const PITCH_LABEL_OFFSET = 36;
const PITCH_CLIP_CLEARANCE = 25;
const CHEVRON_HALF_WIDTH = 22;
const CHEVRON_HEIGHT = 9;
const WARNING_ANGLES = [60, 70, 80, 90, -40, -50, -60, -70, -80, -90];

// Positive pitch lowers the world. Positive bank rotates it counterclockwise.
export function attitudeTransforms({ pitch_deg, roll_deg }, layout = { cx: 0, cy: 0, pitchScale: 4 }) {
  return {
    rotation: `rotate(${-roll_deg} ${layout.cx} ${layout.cy})`,
    translation: `translate(0 ${layout.pitchScale * pitch_deg})`,
  };
}

export function createHorizon(instrument, layers) {
  const get = selector => instrument.querySelector(selector);
  const world = get('.moving-world');
  const rotation = get('#world-rotation');
  const pitch = get('#world-pitch');
  const warningOrigin = get('#warning-origin');
  const warningRotation = get('#warning-rotation');
  const warningPitch = get('#warning-pitch');
  const pointerOrigin = get('#bank-pointer-origin');
  const scaleRotation = get('#scale-rotation');
  const scalePitch = get('#scale-pitch');
  const pointer = get('#bank-pointer');
  layers.background.append(world);
  layers.symbols.append(get('#bank-pointer-clip'));
  layers.warnings.append(warningOrigin);
  const pitchTicks = [];
  const pitchWarnings = [];
  for (let degrees = -90; degrees <= 90; degrees += 2.5) {
    if (degrees === 0) continue;
    const line = svgElement('path', { 'data-pitch': degrees });
    get('#pitch-ladder').append(line);
    const labels = [];
    if (degrees % 10 === 0) {
      for (const [x, anchor] of [[-PITCH_LABEL_OFFSET, 'end'], [PITCH_LABEL_OFFSET, 'start']]) {
        const label = svgElement('text', {}, Math.abs(degrees));
        get('#pitch-ladder').append(label);
        labels.push([label, x, anchor]);
      }
    }
    pitchTicks.push({ degrees, line, labels });
  }
  for (const degrees of WARNING_ANGLES) {
    const path = svgElement('path', { 'data-pitch': degrees });
    get('#pitch-warnings').append(path);
    pitchWarnings.push({ degrees, path });
  }
  let localLayout;
  function resize(layout) {
    const { width, height, attitude: a, pitchScale: p, bankRadius: radius, labelSize } = layout;
    const cx = 0, cy = 0;
    localLayout = { cx, cy, pitchScale: p };
    world.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
    attributes(get('#background-clip rect'), layout.background);
    attributes(get('#attitude-clip rect'), a);
    warningOrigin.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
    pointerOrigin.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
    attributes(get('#pitch-clip rect'), { x: a.x - layout.cx, y: -radius + PITCH_CLIP_CLEARANCE,
      width: a.width, height: a.y + a.height - (layout.cy - radius + PITCH_CLIP_CLEARANCE) });
    // World extent covers every rotation and pitch displacement in the contract.
    const extent = Math.hypot(width, height) + 90 * p;
    get('.sky').setAttribute('d', `M ${cx - extent} ${cy - extent} H ${cx + extent} V ${cy} H ${cx - extent} Z`);
    get('.ground').setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent} V ${cy + extent} H ${cx - extent} Z`);
    get('.horizon-line').setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent}`);
    for (const { degrees, line, labels } of pitchTicks) {
      const major = degrees % 10 === 0;
      const half = major ? PITCH_MAJOR_HALF : degrees % 5 === 0 ? PITCH_MEDIUM_HALF : PITCH_MINOR_HALF;
      const y = -degrees * p;
      line.setAttribute('d', `M ${-half} ${y} H ${half}`);
      for (const [label, x, anchor] of labels) {
        attributes(label, { x, y, 'text-anchor': anchor,
          'dominant-baseline': 'middle', 'font-size': labelSize });
      }
    }
    for (const { degrees, path } of pitchWarnings) {
      const y = -degrees * p;
      const tip = y + (degrees > 0 ? CHEVRON_HEIGHT : -CHEVRON_HEIGHT);
      path.setAttribute('d', `M ${-CHEVRON_HALF_WIDTH} ${y} L 0 ${tip} L ${CHEVRON_HALF_WIDTH} ${y}`);
    }
    pointer.setAttribute('d', `M 0 ${-radius + 2} l -5 9 h 10 Z`);
  }
  function render(frame) {
    const transforms = attitudeTransforms(frame.attitude, localLayout);
    rotation.setAttribute('transform', transforms.rotation);
    pitch.setAttribute('transform', transforms.translation);
    scaleRotation.setAttribute('transform', transforms.rotation);
    scalePitch.setAttribute('transform', transforms.translation);
    pointer.setAttribute('transform', transforms.rotation);
    warningRotation.setAttribute('transform', transforms.rotation);
    warningPitch.setAttribute('transform', transforms.translation);
  }
  return { resize, render };
}

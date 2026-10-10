import { svgElement, setAttributes } from '../helpers.js';

const PITCH_MAJOR_HALF = 30;
const PITCH_MEDIUM_HALF = 16;
const PITCH_MINOR_HALF = 8;
const PITCH_LABEL_OFFSET = 36;
const CHEVRON_HALF_WIDTH = 22;
const CHEVRON_HEIGHT = 9;
const WARNING_ANGLES = [60, 70, 80, 90, -40, -50, -60, -70, -80, -90];

export function createPitchScale(rotation, warningRotation) {
  const pitch = rotation.querySelector('#scale-pitch');
  const ladder = pitch.querySelector('#pitch-ladder');
  const warningPitch = warningRotation.querySelector('#warning-pitch');
  const warnings = warningPitch.querySelector('#pitch-warnings');
  const pitchTicks = [];
  const pitchWarnings = [];
  for (let degrees = -90; degrees <= 90; degrees += 2.5) {
    if (degrees === 0) continue;
    const line = svgElement('path', { 'data-pitch': degrees });
    ladder.append(line);
    const labels = [];
    if (degrees % 10 === 0) {
      for (const [x, anchor] of [[-PITCH_LABEL_OFFSET, 'end'], [PITCH_LABEL_OFFSET, 'start']]) {
        const label = svgElement('text', {}, Math.abs(degrees));
        ladder.append(label);
        labels.push([label, x, anchor]);
      }
    }
    pitchTicks.push({ degrees, line, labels });
  }
  for (const degrees of WARNING_ANGLES) {
    const path = svgElement('path', { 'data-pitch': degrees });
    warnings.append(path);
    pitchWarnings.push({ degrees, path });
  }
  return {
    resize({ pitchScale: pitchPixelsPerDegree, labelSize }) {
      for (const { degrees, line, labels } of pitchTicks) {
        const major = degrees % 10 === 0;
        const half = major ? PITCH_MAJOR_HALF : degrees % 5 === 0 ? PITCH_MEDIUM_HALF : PITCH_MINOR_HALF;
        const y = -degrees * pitchPixelsPerDegree;
        line.setAttribute('d', `M ${-half} ${y} H ${half}`);
        for (const [label, x, anchor] of labels) {
          setAttributes(label, { x, y, 'text-anchor': anchor,
            'dominant-baseline': 'middle', 'font-size': labelSize });
        }
      }
      for (const { degrees, path } of pitchWarnings) {
        const y = -degrees * pitchPixelsPerDegree;
        const tip = y + (degrees > 0 ? CHEVRON_HEIGHT : -CHEVRON_HEIGHT);
        path.setAttribute('d', `M ${-CHEVRON_HALF_WIDTH} ${y} L 0 ${tip} L ${CHEVRON_HALF_WIDTH} ${y}`);
      }
    },
    render(transforms) {
      rotation.setAttribute('transform', transforms.rotation);
      pitch.setAttribute('transform', transforms.translation);
      warningRotation.setAttribute('transform', transforms.rotation);
      warningPitch.setAttribute('transform', transforms.translation);
    },
  };
}

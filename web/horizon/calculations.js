// Positive pitch lowers the world. Positive bank rotates it counterclockwise.
export function attitudeTransforms({ pitch_deg, roll_deg }, layout = { cx: 0, cy: 0, pitchScale: 4 }) {
  return {
    rotation: `rotate(${-roll_deg} ${layout.cx} ${layout.cy})`,
    translation: `translate(0 ${layout.pitchScale * pitch_deg})`,
  };
}

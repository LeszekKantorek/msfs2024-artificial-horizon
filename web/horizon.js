// SVG coordinates: positive pitch lowers the world; positive bank turns it left.
export function attitudeTransforms({ pitch_deg, roll_deg }, layout = { cx: 160, cy: 160, pitchScale: 4 }) {
  return {
    rotation: `rotate(${-roll_deg} ${layout.cx} ${layout.cy})`,
    translation: `translate(0 ${layout.pitchScale * pitch_deg})`,
  };
}

// One latest sample and one pending frame, independently of DOM and transport.
export function createHorizonRenderer({
  draw, obscure, now = () => performance.now(),
  requestFrame = fn => requestAnimationFrame(fn),
  cancelFrame = id => cancelAnimationFrame(id),
}) {
  let latest = null;
  let frame = null;
  let layout;
  let generation = 0;
  function clear() {
    generation++;
    if (frame !== null) cancelFrame(frame);
    frame = null;
    latest = null;
  }
  function schedule() {
    if (frame !== null || !latest) return;
    const current = generation;
    frame = requestFrame(() => {
      if (current !== generation) return;
      frame = null;
      if (!latest) return;
      if (now() >= latest.expiresAt) { clear(); obscure('stale'); return; }
      draw(attitudeTransforms(latest.attitude, layout), latest, layout);
    });
  }
  return {
    accept(sample) { latest = sample; schedule(); },
    resize(next) { layout = next; schedule(); },
    invalidate(reason) { clear(); obscure(reason); },
  };
}

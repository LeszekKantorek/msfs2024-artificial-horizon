// SVG coordinates: positive pitch lowers the world; positive bank turns it left.
export function attitudeTransforms({ pitch_deg, roll_deg }) {
  return {
    rotation: `rotate(${-roll_deg} 160 160)`,
    translation: `translate(0 ${4 * pitch_deg})`,
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
  function clear() {
    if (frame !== null) cancelFrame(frame);
    frame = null;
    latest = null;
  }
  return {
    accept(sample) {
      latest = sample;
      if (frame !== null) return;
      frame = requestFrame(() => {
        frame = null;
        const sample = latest;
        latest = null;
        if (!sample) return;
        if (now() >= sample.expiresAt) { obscure('stale'); return; }
        draw(attitudeTransforms(sample.attitude));
      });
    },
    invalidate(reason) { clear(); obscure(reason); },
  };
}

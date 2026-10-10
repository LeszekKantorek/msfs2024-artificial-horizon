// One latest frame and one pending draw. The panel owns geometry and DOM updates.
export function createFrameScheduler({
  panel, now = () => performance.now(),
  requestFrame = fn => requestAnimationFrame(fn),
  cancelFrame = id => cancelAnimationFrame(id),
}) {
  let latest = null;
  let pending = null;
  let generation = 0;
  function invalidate(reason) {
    generation++;
    if (pending !== null) cancelFrame(pending);
    pending = null;
    latest = null;
    panel.invalidate(reason);
  }
  function schedule() {
    if (pending !== null || !latest) return;
    const current = generation;
    pending = requestFrame(() => {
      if (current !== generation) return;
      pending = null;
      if (!latest) return;
      if (now() >= latest.expiresAt) { invalidate('stale'); return; }
      panel.render(latest);
    });
  }
  return {
    accept(frame) { latest = frame; schedule(); },
    resize(layout) { panel.resize(layout); schedule(); },
    invalidate,
  };
}

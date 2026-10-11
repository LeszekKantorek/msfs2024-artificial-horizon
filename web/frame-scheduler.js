// One latest frame and one pending draw. The panel owns geometry and DOM updates.
export function createFrameScheduler({
  panel, now = () => performance.now(),
  requestFrame = fn => requestAnimationFrame(fn),
  cancelFrame = id => cancelAnimationFrame(id),
  setTimer = (fn, ms) => setTimeout(fn, ms),
  clearTimer = id => clearTimeout(id),
}) {
  let latest = null;
  let pending = null;
  let pendingLayout = null;
  let generation = 0;
  let timer = null;
  let timerGeneration = 0;
  function cancelTimer() {
    timerGeneration++;
    if (timer !== null) clearTimer(timer);
    timer = null;
  }
  function applyLayout() {
    if (!pendingLayout) return;
    const layout = pendingLayout;
    pendingLayout = null;
    panel.resize(layout);
  }
  function invalidate(reason) {
    generation++;
    cancelTimer();
    if (pending !== null) cancelFrame(pending);
    pending = null;
    latest = null;
    panel.invalidate(reason);
    applyLayout();
  }
  function schedule() {
    if (pending !== null || !latest) return;
    const current = generation;
    pending = requestFrame(() => {
      if (current !== generation) return;
      pending = null;
      if (!latest) return;
      const time = now();
      if (time >= latest.expiresAt) { invalidate('stale'); return; }
      applyLayout();
      cancelTimer();
      const nextUpdateAt = panel.render(latest, time);
      if (Number.isFinite(nextUpdateAt)) {
        const token = timerGeneration;
        timer = setTimer(() => {
          if (current !== generation || token !== timerGeneration) return;
          timer = null;
          schedule();
        }, Math.max(0, Math.min(nextUpdateAt, latest.expiresAt) - now()));
      }
    });
  }
  return {
    accept(frame) {
      const time = now();
      if (time >= frame.expiresAt) { invalidate('stale'); return; }
      // Expiry callbacks can be delayed. Recovery must reset the previous history.
      if (latest && time >= latest.expiresAt) invalidate('stale');
      cancelTimer();
      panel.accept?.(frame, time);
      latest = frame; schedule();
    },
    resize(layout) {
      pendingLayout = layout;
      if (latest) schedule(); else applyLayout();
    },
    invalidate,
  };
}

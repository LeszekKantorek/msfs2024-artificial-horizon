const transportLabels = {
  connecting: 'Connecting…', connected: 'Connected',
  reconnecting: 'Connection lost. Retrying…', suspended: 'Page suspended',
};
const sourceLabels = {
  waiting: 'Waiting for fresh telemetry', live: 'Live telemetry', paused: 'Source paused',
  stale: 'Telemetry stale', disconnected: 'Source disconnected', invalid: 'Invalid telemetry',
};
function setText(element, text) {
  // Announce state changes instead of every telemetry sample.
  if (element.textContent !== text) element.textContent = text;
}

export function createStatusView(document) {
  const announcement = document.querySelector('#telemetry-status');
  const instrument = document.querySelector('.instrument');
  const overlay = document.querySelector('#unavailable');
  let currentStatus;
  function announce() {
    setText(announcement, `${sourceLabels[currentStatus.state]}. ${transportLabels[currentStatus.transport]}`);
  }
  function update(status) {
    currentStatus = { ...status };
    instrument.dataset.source = status.source ?? 'unavailable';
    instrument.dataset.transport = status.transport;
    instrument.dataset.state = status.state;
    // No visible success labels. Preserve separate source/transport state for accessibility.
    announce();
  }
  return {
    update,
    render(frame) { update(frame.status); instrument.dataset.available = 'true'; },
    invalidate(reason) {
      instrument.dataset.available = 'false';
      const message = sourceLabels[reason] ?? transportLabels[reason];
      setText(overlay, message);
      // Scheduler expiry can precede the telemetry timer. Keep one consistent announcement.
      if (currentStatus) {
        if (sourceLabels[reason]) {
          currentStatus.state = reason;
          instrument.dataset.state = reason;
        }
        announce();
      } else setText(announcement, message);
    },
  };
}

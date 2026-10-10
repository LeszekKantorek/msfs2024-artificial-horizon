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
  const source = document.querySelector('.source');
  const badge = document.querySelector('#source-badge');
  const background = badge.querySelector('.instrument-background');
  const announcement = document.querySelector('#telemetry-status');
  const instrument = document.querySelector('.instrument');
  const overlay = document.querySelector('#unavailable');
  let currentStatus;
  function announce() {
    setText(announcement, `${currentStatus.source === 'demo' ? 'DEMO. ' : ''}${sourceLabels[currentStatus.state]}. ${transportLabels[currentStatus.transport]}`);
  }
  function update(status) {
    currentStatus = { ...status };
    instrument.dataset.source = status.source ?? 'unavailable';
    instrument.dataset.transport = status.transport;
    instrument.dataset.state = status.state;
    if (instrument.dataset.fixture !== 'true') {
      setText(source, status.source === 'demo' ? 'DEMO' : '');
      badge.style.display = status.source === 'demo' ? '' : 'none';
    }
    // No visible success labels. Preserve separate source/transport state for accessibility.
    announce();
  }
  return {
    update,
    resize(layout) {
      const fixture = instrument.dataset.fixture === 'true';
      badge.setAttribute('transform', `translate(6 ${layout.height - 36})`);
      source.setAttribute('x', 3);
      source.setAttribute('y', 0);
      background.setAttribute('x', 0);
      background.setAttribute('y', fixture ? -16 : -10);
      background.setAttribute('width', 46);
      background.setAttribute('height', fixture ? 30 : 20);
    },
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

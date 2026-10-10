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
  const transport = document.querySelector('#transport');
  const data = document.querySelector('#data-status');
  const instrument = document.querySelector('.instrument');
  const overlay = document.querySelector('#unavailable');
  function update(status) {
    setText(source, status.source === 'demo' ? 'DEMO' :
      status.source === 'simconnect' ? 'SIMCONNECT' : 'Source unavailable');
    setText(transport, transportLabels[status.transport]);
    setText(data, sourceLabels[status.state]);
    data.dataset.state = status.state;
  }
  return {
    update,
    render(frame) { update(frame.status); instrument.dataset.available = 'true'; },
    invalidate(reason) {
      instrument.dataset.available = 'false';
      setText(overlay, sourceLabels[reason] ?? transportLabels[reason]);
      if (reason === 'stale') { setText(data, sourceLabels.stale); data.dataset.state = 'stale'; }
    },
  };
}

import { createTelemetryClient } from './telemetry-client.js';

const source = document.querySelector('.source');
const transport = document.querySelector('#transport');
const data = document.querySelector('#data-status');
const transportLabels = {
  connecting: 'Connecting…', connected: 'Connected',
  reconnecting: 'Connection lost. Retrying every 2 seconds…', suspended: 'Page suspended',
};
const sourceLabels = {
  waiting: 'Waiting for fresh telemetry', live: 'Live telemetry', paused: 'Source paused',
  stale: 'Telemetry stale', disconnected: 'Source disconnected', invalid: 'Invalid telemetry',
};
function setText(element, text) {
  // Do not announce identical status at the telemetry cadence.
  if (element.textContent !== text) element.textContent = text;
}
const client = createTelemetryClient({ onStatus(status) {
  setText(source, status.source === 'demo' ? 'DEMO' :
    status.source === 'simconnect' ? 'SIMCONNECT' : 'Source unavailable');
  setText(transport, transportLabels[status.transport]);
  setText(data, sourceLabels[status.state]);
  data.dataset.state = status.state;
} });
window.addEventListener('pagehide', () => client.stop());
window.addEventListener('pageshow', () => { if (!document.hidden) client.start(); });
document.addEventListener('visibilitychange', () => {
  if (document.hidden) client.stop(); else client.start();
});
if (!document.hidden) client.start();

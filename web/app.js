import { createTelemetryClient } from './telemetry-client.js';
import { createHorizonRenderer } from './horizon.js';
import { pfdLayout } from './layout.js';
import { createPfdView } from './pfd-view.js';

const source = document.querySelector('.source');
const transport = document.querySelector('#transport');
const data = document.querySelector('#data-status');
const instrument = document.querySelector('.instrument');
const overlay = document.querySelector('#unavailable');
const svg = instrument.querySelector('svg');
const view = createPfdView(instrument);
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
const renderer = createHorizonRenderer({
  draw: view.draw,
  obscure(reason) {
    view.obscure();
    setText(overlay, sourceLabels[reason] ?? transportLabels[reason]);
    if (reason === 'stale') {
      setText(data, sourceLabels.stale);
      data.dataset.state = 'stale';
    }
  },
});
function resize() {
  const { width, height } = svg.getBoundingClientRect();
  if (width <= 0 || height <= 0) return;
  const layout = pfdLayout(width, height);
  if (instrument.dataset.available === 'false') view.resize(layout);
  renderer.resize(layout);
}
new ResizeObserver(resize).observe(svg);
resize();
const client = createTelemetryClient({
  onSample: sample => renderer.accept(sample),
  onStatus(status) {
    setText(source, status.source === 'demo' ? 'DEMO' :
      status.source === 'simconnect' ? 'SIMCONNECT' : 'Source unavailable');
    setText(transport, transportLabels[status.transport]);
    setText(data, sourceLabels[status.state]);
    data.dataset.state = status.state;
    if (status.state !== 'live') {
      renderer.invalidate(status.transport === 'connected' ? status.state : status.transport);
    }
  },
});
window.addEventListener('pagehide', () => client.stop());
window.addEventListener('pageshow', () => { if (!document.hidden) client.start(); });
document.addEventListener('visibilitychange', () => {
  if (document.hidden) client.stop(); else client.start();
});
if (!document.hidden) client.start();

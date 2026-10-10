import { createTelemetryClient } from './telemetry-client.js';
import { createFrameScheduler } from './frame-scheduler.js';
import { pfdLayout } from './layout.js';
import { createPanel } from './panel.js';
import { createStatusView } from './status.js';

const instrument = document.querySelector('.instrument');
const svg = instrument.querySelector('svg');
const statusView = createStatusView(document);
const panel = createPanel(instrument, statusView);
const scheduler = createFrameScheduler({ panel });
let status;
function resize() {
  const { width, height } = svg.getBoundingClientRect();
  if (width <= 0 || height <= 0) return;
  scheduler.resize(pfdLayout(width, height));
}
new ResizeObserver(resize).observe(svg);
resize();
const client = createTelemetryClient({
  onSample: sample => scheduler.accept({ ...sample, status: { ...status } }),
  onStatus(next) {
    status = next;
    statusView.update(status);
    if (status.state !== 'live') {
      scheduler.invalidate(status.transport === 'connected' ? status.state : status.transport);
    }
  },
});
window.addEventListener('pagehide', () => client.stop());
window.addEventListener('pageshow', () => { if (!document.hidden) client.start(); });
document.addEventListener('visibilitychange', () => {
  if (document.hidden) client.stop(); else client.start();
});
if (!document.hidden) client.start();

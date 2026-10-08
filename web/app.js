import { createTelemetryClient } from './telemetry-client.js';
import { createHorizonRenderer } from './horizon.js';

const source = document.querySelector('.source');
const transport = document.querySelector('#transport');
const data = document.querySelector('#data-status');
const instrument = document.querySelector('.instrument');
const overlay = document.querySelector('#unavailable');
const rotation = document.querySelector('#world-rotation');
const translation = document.querySelector('#world-pitch');
const bankPointer = document.querySelector('#bank-pointer');
const transportLabels = {
  connecting: 'Connecting…', connected: 'Connected',
  reconnecting: 'Connection lost. Retrying…', suspended: 'Page suspended',
};
const sourceLabels = {
  waiting: 'Waiting for fresh telemetry', live: 'Live telemetry', paused: 'Source paused',
  stale: 'Telemetry stale', disconnected: 'Source disconnected', invalid: 'Invalid telemetry',
};
function setText(element, text) {
  // Screen readers announce state changes, not every telemetry sample.
  if (element.textContent !== text) element.textContent = text;
}
const renderer = createHorizonRenderer({
  draw(transforms) {
    rotation.setAttribute('transform', transforms.rotation);
    translation.setAttribute('transform', transforms.translation);
    bankPointer.setAttribute('transform', transforms.rotation);
    instrument.dataset.available = 'true';
  },
  obscure(reason) {
    instrument.dataset.available = 'false';
    setText(overlay, sourceLabels[reason] ?? transportLabels[reason]);
    if (reason === 'stale') {
      setText(data, sourceLabels.stale);
      data.dataset.state = 'stale';
    }
  },
});

// Static scales are generated once; only group transforms change during live rendering.
const svgNS = 'http://www.w3.org/2000/svg';
function svgElement(name, attributes, text) {
  const element = document.createElementNS(svgNS, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  if (text !== undefined) element.textContent = text;
  return element;
}
const ladder = document.querySelector('#pitch-ladder');
for (let degrees = -90; degrees <= 90; degrees += 5) {
  if (degrees === 0) continue;
  const y = 160 - degrees * 4;
  const major = degrees % 10 === 0;
  const halfWidth = major ? 32 : 16;
  ladder.append(svgElement('path', { d: `M ${160 - halfWidth} ${y} H ${160 + halfWidth}` }));
  if (major) {
    for (const [x, anchor] of [[119, 'end'], [201, 'start']]) {
      ladder.append(svgElement('text', { x, y, 'text-anchor': anchor, dy: '0.35em' }, Math.abs(degrees)));
    }
  }
}
const bankScale = document.querySelector('#bank-scale');
for (const degrees of [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60]) {
  bankScale.append(svgElement('path', {
    d: `M 160 22 V ${degrees % 30 === 0 ? 38 : 31}`,
    transform: `rotate(${degrees} 160 160)`,
  }));
}
const client = createTelemetryClient({
  onAttitude: sample => renderer.accept(sample),
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

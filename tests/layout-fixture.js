// Development only: the complete intended arrangement, without new telemetry fields.
import { pfdLayout } from '/layout.js';
import { svgElement } from '/svg.js';

const svg = document.querySelector('svg');
const layer = document.querySelector('#layout-fixtures');
document.querySelector('#instrument-title').textContent = 'Development layout fixture';
const badge = document.createElement('span');
badge.className = 'source';
badge.textContent = 'LAYOUT FIXTURE';
document.querySelector('.source').replaceWith(badge);

function text(parent, x, y, value, size = 14, color = '#fff') {
  parent.append(svgElement('text', { x, y, fill: color, 'font-size': size,
    'text-anchor': 'middle', 'dominant-baseline': 'middle' }, value));
}
function render() {
  const { width, height } = svg.getBoundingClientRect();
  if (width <= 0 || height <= 0) return;
  const layout = pfdLayout(width, height);
  layer.replaceChildren();
  const regions = [layout.airspeed, layout.altitude, layout.vsi, layout.heading];
  for (const r of regions) {
    layer.append(svgElement('rect', { ...r, fill: '#18232c' }));
  }
  const { airspeed: a, altitude: b, vsi: v, heading: h } = layout;
  for (const [r, labels] of [[a, ['160', '140', '120', '100', '80']],
    [b, ['10200', '10100', '10000', '9900', '9800']]]) {
    labels.forEach((value, i) => text(layer, r.x + r.width / 2,
      r.y + r.height * (i + 0.5) / 5, value, 12));
  }
  for (const [r, value] of [[a, '123'], [b, '10000']]) {
    layer.append(svgElement('rect', { x: r.x + 1, y: layout.cy - 14,
      width: r.width - 2, height: 28, fill: '#080d12', stroke: '#fff' }));
    text(layer, r.x + r.width / 2, layout.cy, value, 19);
  }
  layer.append(svgElement('rect', { x: a.x + a.width - 6, y: a.y + 12,
    width: 4, height: a.height - 24, fill: '#73d39e' }));
  layer.append(svgElement('path', { d: `M ${a.x + a.width - 2} ${layout.cy} v ${-a.height / 4}`,
    stroke: '#ed83e7', 'stroke-width': 2 }));
  text(layer, a.x + a.width - 9, a.y + 24, 'V', 12, '#86def5');
  text(layer, a.x + a.width / 2, height - 15, 'GS 123', 12);
  text(layer, b.x + b.width / 2, 16, '12000', 15, '#86def5');
  text(layer, b.x + b.width / 2, height - 15, '29.92', 13, '#86def5');
  layer.append(svgElement('path', { d: `M ${b.x} ${layout.cy - 30} l 10 -5 v 10 Z`, fill: '#86def5' }));
  text(layer, v.x + v.width / 2, v.y + 16, '2', 12);
  text(layer, v.x + v.width / 2, layout.cy, '0', 12);
  text(layer, v.x + v.width / 2, v.y + v.height - 16, '2', 12);
  layer.append(svgElement('path', { d: `M ${v.x + 2} ${layout.cy - 30} l 10 -5 v 10 Z`, fill: '#fff' }));
  text(layer, h.x + 24, 16, '270', 12);
  text(layer, layout.cx, 16, '280°', 19);
  text(layer, h.x + h.width - 24, 16, '290', 12);
  layer.append(svgElement('path', { d: `M ${layout.cx + 32} 2 l -5 8 h 10 Z`, fill: '#ed83e7' }));
  text(layer, a.x + a.width / 2, 16, '300°', 13, '#86def5');
  layer.dataset.items = '1,2,3,4,5,7,8,9,10,11,12,13,15,16,18,19,20,21';
}
new ResizeObserver(render).observe(svg);

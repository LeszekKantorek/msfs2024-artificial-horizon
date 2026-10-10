// Development only: implemented instruments plus illustrations of future features.
import { pfdLayout } from '/layout.js';
import { svgElement } from '/svg.js';

const svg = document.querySelector('svg');
const instrument = document.querySelector('.instrument');
instrument.dataset.fixture = 'true';
document.querySelector('#instrument-title').textContent = 'Development layout fixture';
const candidate = Number(new URLSearchParams(location.search).get('opacity'));
if ([0.50, 0.65, 0.80].includes(candidate)) {
  instrument.style.setProperty('--instrument-background-opacity', candidate);
}
const layers = {
  airspeed: document.querySelector('#layer-airspeed'),
  altitude: document.querySelector('#layer-altitude'),
  symbols: document.querySelector('#layer-symbols'),
};
const badge = svgElement('g', { id: 'layout-badge' });
badge.append(svgElement('rect', { x: 0, y: -12, width: 37, height: 22, class: 'instrument-background' }));
const label = svgElement('text', { class: 'fixture-label' });
label.append(svgElement('tspan', { x: 3, y: -5 }, 'LAYOUT'), svgElement('tspan', { x: 3, y: 4 }, 'FIXTURE'));
badge.append(label);
layers.symbols.append(badge);
const marker = svgElement('g', { id: 'layout-fixtures' });
layers.symbols.append(marker);
const groups = {};
for (const [name, parent] of [['airspeed', layers.airspeed], ['altitude', layers.altitude],
  ['vsi', layers.altitude], ['heading', layers.symbols], ['supplemental', layers.symbols]]) {
  const group = svgElement('g', { id: `fixture-${name}` });
  parent.append(group); groups[name] = group;
}

function text(parent, x, y, value, size = 14, color = '#fff') {
  parent.append(svgElement('text', { x, y, fill: color, 'font-size': size,
    'text-anchor': 'middle', 'dominant-baseline': 'middle' }, value));
}
function render() {
  const { width, height } = svg.getBoundingClientRect();
  if (width <= 0 || height <= 0) return;
  const layout = pfdLayout(width, height);
  badge.setAttribute('transform', `translate(6 ${height - 36})`);
  for (const group of Object.values(groups)) group.replaceChildren();
  const layer = groups.supplemental;
  for (const name of ['altitude', 'vsi', 'heading']) {
    groups[name].append(svgElement('rect', { ...layout[name], class: 'instrument-background' }));
  }
  const { airspeed: a, altitude: b, vsi: v, heading: h } = layout;
  for (const [r, labels] of [[b, ['10200', '10100', '10000', '9900', '9800']]]) {
    const parent = r === a ? groups.airspeed : groups.altitude;
    labels.forEach((value, i) => text(parent, r.x + r.width / 2,
      r.y + r.height * (i + 0.5) / 5, value, 12));
  }
  for (const [r, value] of [[b, '10000']]) {
    const parent = r === a ? groups.airspeed : groups.altitude;
    parent.append(svgElement('rect', { x: r.x + 1, y: layout.cy - 14,
      width: r.width - 2, height: 28, fill: '#080d12', stroke: '#fff' }));
    text(parent, r.x + r.width / 2, layout.cy, value, 19);
  }
  groups.airspeed.append(svgElement('rect', { x: a.x + a.width - 6, y: a.y + 12,
    width: 2, height: a.height - 24, fill: '#73d39e' }));
  groups.airspeed.append(svgElement('path', { d: `M ${a.x + a.width - 1} ${layout.cy - 18} v ${-a.height / 4}`,
    stroke: '#ed83e7', 'stroke-width': 2 }));
  text(groups.airspeed, a.x + a.width - 4, a.y + 22, 'V', 8, '#86def5');
  layer.append(svgElement('rect', { x: b.x, y: 0, width: b.width, height: 32, class: 'instrument-background' }));
  layer.append(svgElement('rect', { x: b.x, y: height - 24, width: b.width, height: 22, class: 'instrument-background' }));
  layer.append(svgElement('rect', { x: a.x, y: 0, width: a.width, height: 32, class: 'instrument-background' }));
  text(layer, b.x + b.width / 2, 16, '12000', 15, '#86def5');
  text(layer, b.x + b.width / 2, height - 15, '29.92', 13, '#86def5');
  groups.altitude.append(svgElement('path', { d: `M ${b.x} ${layout.cy - 30} l 10 -5 v 10 Z`, fill: '#86def5' }));
  text(groups.vsi, v.x + v.width / 2, v.y + 16, '2', 12);
  text(groups.vsi, v.x + v.width / 2, layout.cy, '0', 12);
  text(groups.vsi, v.x + v.width / 2, v.y + v.height - 16, '2', 12);
  groups.vsi.append(svgElement('path', { d: `M ${v.x + 2} ${layout.cy - 30} l 10 -5 v 10 Z`, fill: '#fff' }));
  text(groups.heading, h.x + 24, 16, '270', 12);
  text(groups.heading, layout.cx, 16, '280°', 19);
  text(groups.heading, h.x + h.width - 24, 16, '290', 12);
  groups.heading.append(svgElement('path', { d: `M ${layout.cx + 32} 2 l -5 8 h 10 Z`, fill: '#ed83e7' }));
  text(layer, a.x + a.width / 2, 16, '300°', 13, '#86def5');
  marker.dataset.items = '1,2,3,4,5,7,8,9,10,11,12,13,15,16,18,19,20,21';
}
new ResizeObserver(render).observe(svg);

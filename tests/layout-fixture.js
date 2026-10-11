// Development only: implemented instruments plus illustrations of future features.
import { pfdLayout } from '/layout.js';
import { svgElement } from '/helpers.js';

const svg = document.querySelector('svg');
const instrument = document.querySelector('.instrument');
instrument.dataset.fixture = 'true';
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
  ['heading', layers.symbols], ['supplemental', layers.symbols]]) {
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
  groups.heading.append(svgElement('rect', { ...layout.heading, class: 'instrument-background' }));
  const { airspeed: a, heading: h } = layout;
  layer.append(svgElement('rect', { x: a.x, y: 0, width: a.width, height: 32, class: 'instrument-background' }));
  text(groups.heading, h.x + 24, 16, '270', 12);
  text(groups.heading, layout.cx, 16, '280°', 19);
  text(groups.heading, h.x + h.width - 24, 16, '290', 12);
  groups.heading.append(svgElement('path', { d: `M ${layout.cx + 32} 2 l -5 8 h 10 Z`, fill: '#ed83e7' }));
  text(layer, a.x + a.width / 2, 16, '300°', 13, '#86def5');
  marker.dataset.items = '1,2,3,4,5,7,8,9,10,11,12,13,15,16,18,19,20,21';
}
new ResizeObserver(render).observe(svg);

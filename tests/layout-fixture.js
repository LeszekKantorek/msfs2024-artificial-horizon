// Development only: full panel built from production instruments.
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
function render() {
  const { width, height } = svg.getBoundingClientRect();
  if (width <= 0 || height <= 0) return;
  badge.setAttribute('transform', `translate(6 ${height - 36})`);
  marker.dataset.items = '1,2,3,4,5,7,8,9,10,11,12,13,15,16,18,20,21';
}
new ResizeObserver(render).observe(svg);

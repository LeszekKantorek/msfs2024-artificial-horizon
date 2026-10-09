export function svgElement(name, attributes, text) {
  const element = document.createElementNS('http://www.w3.org/2000/svg', name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  if (text !== undefined) element.textContent = text;
  return element;
}
function attributes(element, values) {
  for (const [key, value] of Object.entries(values)) element.setAttribute(key, value);
}

// Owns SVG construction and DOM updates; layout and telemetry have no DOM dependency.
export function createPfdView(instrument) {
  const get = selector => instrument.querySelector(selector);
  const svg = get('svg');
  const rotation = get('#world-rotation');
  const pitch = get('#world-pitch');
  const pointer = get('#bank-pointer');
  const slip = get('#slip-skid');
  const turn = get('#turn-rate');
  let layout = null;

  function resize(next) {
    layout = next;
    const { width, height, cx, cy, attitude: a, pitchScale: p, bankRadius: radius,
      aircraftWidth, labelSize, slipY, slipTravel, turnY, turnTravel } = layout;
    svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
    attributes(get('#attitude-clip rect'), a);
    attributes(get('#pitch-clip rect'), { x: a.x, y: cy - radius + 25,
      width: a.width, height: a.y + a.height - (cy - radius + 25) });
    // World extent covers every rotation and pitch displacement in the contract.
    const extent = Math.hypot(width, height) + 90 * p;
    get('.sky').setAttribute('d', `M ${cx - extent} ${cy - extent} H ${cx + extent} V ${cy} H ${cx - extent} Z`);
    get('.ground').setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent} V ${cy + extent} H ${cx - extent} Z`);
    get('.horizon-line').setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent}`);
    const ladder = get('#pitch-ladder');
    ladder.replaceChildren();
    for (let degrees = -90; degrees <= 90; degrees += 2.5) {
      if (degrees === 0) continue;
      const major = degrees % 10 === 0;
      const half = major ? 30 : degrees % 5 === 0 ? 16 : 8;
      const y = cy - degrees * p;
      ladder.append(svgElement('path', { d: `M ${cx - half} ${y} H ${cx + half}`, 'data-pitch': degrees }));
      if (major) {
        for (const [x, anchor] of [[cx - 36, 'end'], [cx + 36, 'start']]) {
          ladder.append(svgElement('text', { x, y, 'text-anchor': anchor,
            'dominant-baseline': 'middle', 'font-size': labelSize }, Math.abs(degrees)));
        }
      }
    }
    const warnings = get('#pitch-warnings');
    warnings.replaceChildren();
    for (const degrees of [60, 70, 80, 90, -40, -50, -60, -70, -80, -90]) {
      const y = cy - degrees * p;
      const tip = y + (degrees > 0 ? 9 : -9);
      warnings.append(svgElement('path', { d: `M ${cx - 22} ${y} L ${cx} ${tip} L ${cx + 22} ${y}`,
        'data-pitch': degrees }));
    }
    const bank = get('#bank-scale');
    bank.replaceChildren(svgElement('path', { class: 'bank-zero',
      d: `M ${cx} ${cy - radius - 10} l -5 -8 h 10 Z` }));
    for (const degrees of [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60]) {
      bank.append(svgElement('path', { d: `M ${cx} ${cy - radius} v ${degrees % 30 === 0 ? -12 : -7}`,
        transform: `rotate(${degrees} ${cx} ${cy})` }));
    }
    pointer.setAttribute('d', `M ${cx} ${cy - radius + 2} l -5 9 h 10 Z`);
    const half = aircraftWidth / 2;
    const aircraft = `M ${cx - half} ${cy} H ${cx - 25} l 12 12 M ${cx + 13} ${cy + 12} l 12 -12 H ${cx + half} M ${cx - 5} ${cy} h 10`;
    get('.aircraft').setAttribute('d', aircraft);
    get('.aircraft-outline').setAttribute('d', aircraft);
    slip.setAttribute('transform', `translate(${cx} ${slipY})`);
    get('.slip-reference').setAttribute('d', `M -9 -7 V 7 M 9 -7 V 7 M ${-slipTravel - 7} 0 h 4 M ${slipTravel + 3} 0 h 4`);
    turn.setAttribute('transform', `translate(${cx} ${turnY})`);
    get('.turn-scale').setAttribute('d', `M ${-turnTravel} 0 H ${turnTravel} M 0 -5 V 5 M ${-turnTravel / 2} -5 V 5 M ${turnTravel / 2} -5 V 5`);
  }
  function draw(transforms, sample, nextLayout) {
    if (nextLayout !== layout) resize(nextLayout);
    rotation.setAttribute('transform', transforms.rotation);
    pitch.setAttribute('transform', transforms.translation);
    get('#scale-rotation').setAttribute('transform', transforms.rotation);
    get('#scale-pitch').setAttribute('transform', transforms.translation);
    pointer.setAttribute('transform', transforms.rotation);
    const slipValue = sample.slip_skid;
    const turnValue = sample.turn_rate_dps;
    slip.dataset.available = String(slipValue !== null);
    turn.dataset.available = String(turnValue !== null);
    slip.setAttribute('aria-label', slipValue === null ? 'Slip/skid unavailable' :
      slipValue === 0 ? 'Slip/skid centered' : `Slip/skid ${slipValue < 0 ? 'left' : 'right'}`);
    turn.setAttribute('aria-label', turnValue === null ? 'Turn rate unavailable' : `Turn rate ${turnValue} degrees per second`);
    if (slipValue !== null) get('.slip-ball').setAttribute('cx', slipValue * layout.slipTravel);
    if (turnValue !== null) {
      const end = Math.max(-6, Math.min(6, turnValue)) / 6 * layout.turnTravel;
      get('.turn-vector').setAttribute('d', `M 0 0 H ${end}`);
      get('.turn-overflow').setAttribute('d', Math.abs(turnValue) > 6 ?
        `M ${end} 0 l ${turnValue > 0 ? -6 : 6} -5 v 10 Z` : '');
    }
    instrument.dataset.available = 'true';
  }
  return { resize, draw, obscure() { instrument.dataset.available = 'false'; } };
}

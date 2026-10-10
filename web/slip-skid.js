// Geometry stays in the indication's local coordinates.
export function slipPosition(value, travel) { return value * travel; }

export function createSlipSkid(instrument, layers) {
  const group = instrument.querySelector('#slip-skid');
  const warning = instrument.querySelector('#slip-warning');
  const background = group.querySelector('.instrument-background');
  layers.symbols.append(group);
  layers.warnings.append(warning);
  const reference = group.querySelector('.slip-reference');
  const ball = group.querySelector('.slip-ball');
  let travel;
  return {
    resize(layout) {
      travel = layout.slipTravel;
      group.setAttribute('transform', `translate(${layout.cx} ${layout.slipY})`);
      warning.setAttribute('transform', group.getAttribute('transform'));
      background.setAttribute('x', -travel - 12);
      background.setAttribute('y', -10);
      background.setAttribute('width', 2 * travel + 24);
      background.setAttribute('height', 20);
      reference.setAttribute('d', `M -9 -7 V 7 M 9 -7 V 7 M ${-travel - 7} 0 h 4 M ${travel + 3} 0 h 4`);
    },
    render(frame) {
      const value = frame.slip_skid;
      const available = value !== null && value !== undefined;
      group.dataset.available = String(available);
      warning.dataset.available = String(available);
      group.setAttribute('aria-label', !available ? 'Slip/skid unavailable' :
        value === 0 ? 'Slip/skid centered' : `Slip/skid ${value < 0 ? 'left' : 'right'}`);
      if (available) ball.setAttribute('cx', slipPosition(value, travel));
    },
  };
}

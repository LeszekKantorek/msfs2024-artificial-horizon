// Geometry stays in the indication's local coordinates.
export function slipPosition(value, travel) { return value * travel; }

export function createSlipSkid(instrument) {
  const group = instrument.querySelector('#slip-skid');
  const reference = group.querySelector('.slip-reference');
  const ball = group.querySelector('.slip-ball');
  let travel;
  return {
    resize(layout) {
      travel = layout.slipTravel;
      group.setAttribute('transform', `translate(${layout.cx} ${layout.slipY})`);
      reference.setAttribute('d', `M -9 -7 V 7 M 9 -7 V 7 M ${-travel - 7} 0 h 4 M ${travel + 3} 0 h 4`);
    },
    render(frame) {
      const value = frame.slip_skid;
      const available = value !== null && value !== undefined;
      group.dataset.available = String(available);
      group.setAttribute('aria-label', !available ? 'Slip/skid unavailable' :
        value === 0 ? 'Slip/skid centered' : `Slip/skid ${value < 0 ? 'left' : 'right'}`);
      if (available) ball.setAttribute('cx', slipPosition(value, travel));
    },
  };
}

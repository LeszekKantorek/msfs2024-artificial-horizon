const DISPLAY_LIMIT = 6;
const STANDARD_RATE = 3;
const MARK_HALF_HEIGHT = 5;
const ARROW_WIDTH = 6;

export function turnGeometry(value, travel) {
  const end = Math.max(-DISPLAY_LIMIT, Math.min(DISPLAY_LIMIT, value)) / DISPLAY_LIMIT * travel;
  return { end, vector: `M 0 0 H ${end}`, overflow: Math.abs(value) > DISPLAY_LIMIT ?
    `M ${end} 0 l ${value > 0 ? -ARROW_WIDTH : ARROW_WIDTH} -${MARK_HALF_HEIGHT} v ${2 * MARK_HALF_HEIGHT} Z` : '' };
}

export function createTurnRate(instrument) {
  const group = instrument.querySelector('#turn-rate');
  const scale = group.querySelector('.turn-scale');
  const vector = group.querySelector('.turn-vector');
  const overflow = group.querySelector('.turn-overflow');
  let travel;
  return {
    resize(layout) {
      travel = layout.turnTravel;
      const standard = STANDARD_RATE / DISPLAY_LIMIT * travel;
      const h = MARK_HALF_HEIGHT;
      group.setAttribute('transform', `translate(${layout.cx} ${layout.turnY})`);
      scale.setAttribute('d', `M ${-travel} 0 H ${travel} M 0 -${h} V ${h} M ${-standard} -${h} V ${h} M ${standard} -${h} V ${h}`);
    },
    render(frame) {
      const value = frame.turn_rate_dps;
      const available = value !== null && value !== undefined;
      group.dataset.available = String(available);
      group.setAttribute('aria-label', !available ? 'Turn rate unavailable' : `Turn rate ${value} degrees per second`);
      if (available) {
        const geometry = turnGeometry(value, travel);
        vector.setAttribute('d', geometry.vector);
        overflow.setAttribute('d', geometry.overflow);
      }
    },
  };
}

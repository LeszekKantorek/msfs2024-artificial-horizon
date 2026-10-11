const PIXELS_PER_KNOT = 4;
export const MAX_SPEED = 999;

// Missing/invalid data and display overflow are different presentation states.
export function speedState(value) {
  if (!Number.isFinite(value) || value < 0) return 'unavailable';
  return value > MAX_SPEED ? 'overflow' : 'available';
}

export function tapeOffset(speed, tick) { return (speed - tick) * PIXELS_PER_KNOT; }

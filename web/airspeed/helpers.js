const PIXELS_PER_KNOT = 4;
export const MAX_SPEED = 999;

// Missing/invalid data and display overflow are different presentation states.
export function speedState(value) {
  if (!Number.isFinite(value) || value < 0) return 'unavailable';
  return value > MAX_SPEED ? 'overflow' : 'available';
}

export function tapeOffset(speed, tick) { return (speed - tick) * PIXELS_PER_KNOT; }

// Dynamic redline and fixed VNE retain distinct meanings, with the lower limit controlling warnings.
export function maximumSpeed(profile) {
  const values = [profile?.vne_kt, profile?.redline_kt].filter(value => Number.isFinite(value) && value > 0);
  return values.length ? Math.min(...values) : null;
}

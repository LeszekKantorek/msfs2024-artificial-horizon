import { clamp } from './helpers.js';

// All coordinates are CSS pixels. Text and symbols do not stretch with the panel.
export function pfdLayout(width, height) {
  const airspeedWidth = clamp(width * 0.16, 50, 84);
  const altitudeWidth = clamp(width * 0.19, 60, 92);
  const verticalSpeedWidth = clamp(width * 0.09, 42, 54);
  const top = 32;
  const bottom = 48;
  const attitude = { x: airspeedWidth, y: top, width: width - airspeedWidth - altitudeWidth - verticalSpeedWidth,
    height: height - top - bottom };
  const cx = attitude.x + attitude.width / 2;
  const cy = attitude.y + attitude.height / 2;
  return {
    width, height, background: { x: 0, y: 0, width, height }, attitude, cx, cy,
    heading: { x: airspeedWidth, y: 0, width: attitude.width, height: top },
    airspeed: { x: 0, y: top, width: airspeedWidth, height: attitude.height },
    altitude: { x: width - altitudeWidth - verticalSpeedWidth, y: top, width: altitudeWidth, height: attitude.height },
    vsi: { x: width - verticalSpeedWidth, y: top, width: verticalSpeedWidth, height: attitude.height },
    pitchScale: Math.max(3.2, attitude.height / 55),
    bankRadius: Math.min(100, attitude.width * 0.42, attitude.height / 2 - 20),
    aircraftWidth: clamp(attitude.width * 0.52, 80, 138),
    labelSize: clamp(width / 30, 12, 15),
    slipY: height - 34,
    slipTravel: Math.min(28, attitude.width * 0.15),
    turnY: height - 12,
    turnTravel: Math.min(80, (attitude.width - 16) / 2),
  };
}

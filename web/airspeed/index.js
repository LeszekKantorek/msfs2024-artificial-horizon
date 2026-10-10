import { calculateAirspeedGeometry, deriveSpeedReadings } from './calculations.js';
import { createIas } from './ias.js';
import { createGroundSpeed } from './ground-speed.js';

export function createAirspeed(instrument, layers) {
  const ias = createIas(instrument.querySelector('defs'), layers.airspeed, layers.warnings);
  const groundSpeed = createGroundSpeed(layers.airspeed, layers.warnings);
  return {
    resize(layout) {
      const geometry = calculateAirspeedGeometry(layout);
      ias.resize(geometry.tape, geometry.readout);
      groundSpeed.resize(geometry.groundSpeed);
    },
    render(frame) {
      const { groundSpeed: gs, ...iasReadings } = deriveSpeedReadings(frame);
      ias.render(iasReadings);
      groundSpeed.render(gs);
    },
  };
}

import { createIas } from './ias.js';
import { createGroundSpeed } from './ground-speed.js';

export function createAirspeed(instrument, layers) {
  const ias = createIas(instrument.querySelector('defs'), layers.airspeed, layers.warnings);
  const groundSpeed = createGroundSpeed(layers.airspeed, layers.warnings);
  return {
    resize(layout) {
      const groundSpeedPanelY = layout.height - 24;
      ias.resize(layout.airspeed, groundSpeedPanelY);
      groundSpeed.resize({ x: layout.airspeed.x, width: layout.airspeed.width, y: groundSpeedPanelY });
    },
    render(frame) {
      ias.render(frame.ias_kt, frame.airspeed_profile, frame.ias_acceleration_ktps);
      groundSpeed.render(frame.gs_kt);
    },
  };
}

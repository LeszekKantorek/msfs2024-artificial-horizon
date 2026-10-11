const STATES = new Set(['waiting', 'live', 'paused', 'stale', 'disconnected', 'invalid']);

export function isValidSnapshot(value) {
  if (!value || value.schema_version !== 1 ||
      !Number.isSafeInteger(value.sequence) || value.sequence < 0 ||
      !['demo', 'simconnect'].includes(value.source) || !STATES.has(value.state) ||
      !(value.sample_age_ms === null ||
        (Number.isInteger(value.sample_age_ms) && value.sample_age_ms >= 0))) return false;
  if (value.state !== 'live') return value.attitude === null;
  const a = value.attitude;
  return value.sample_age_ms !== null && a !== null && typeof a === 'object' &&
    Number.isFinite(a.pitch_deg) && a.pitch_deg >= -90 && a.pitch_deg <= 90 &&
    Number.isFinite(a.roll_deg) && a.roll_deg >= -180 && a.roll_deg < 180;
}

// Optional instruments fail independently of the envelope and attitude.
export function normalizeAirspeedProfile(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const keys = ['vs0_kt', 'vs1_kt', 'vfe_kt', 'vno_kt', 'vne_kt'];
  const thresholds = keys.map(key => value[key]);
  if (thresholds.some((v, i) => !Number.isFinite(v) || v <= 0 || (i > 0 && v <= thresholds[i - 1]))) return null;
  const references = value.references;
  if (!references || typeof references !== 'object' || Array.isArray(references)) return null;
  const refKeys = ['vr_kt', 'vx_kt', 'vy_kt', 'vg_kt'];
  if (refKeys.some(key => !Number.isFinite(references[key]) || references[key] <= 0 || references[key] > value.vne_kt)) return null;
  return { ...Object.fromEntries(keys.map(key => [key, value[key]])),
    references: Object.fromEntries(refKeys.map(key => [key, references[key]])) };
}

export function normalizeOptionalIndications(value) {
  return {
    altitude_ft: Number.isFinite(value.altitude_ft) ? value.altitude_ft : null,
    selected_altitude_ft: Number.isFinite(value.selected_altitude_ft) ? value.selected_altitude_ft : null,
    vertical_speed_fpm: Number.isFinite(value.vertical_speed_fpm) ? value.vertical_speed_fpm : null,
    baro_inhg: Number.isFinite(value.baro_inhg) && value.baro_inhg > 0 ? value.baro_inhg : null,
    slip_skid: Number.isFinite(value.slip_skid) && Math.abs(value.slip_skid) <= 1 ? value.slip_skid : null,
    turn_rate_dps: Number.isFinite(value.turn_rate_dps) ? value.turn_rate_dps : null,
    ias_kt: Number.isFinite(value.ias_kt) && value.ias_kt >= 0 ? value.ias_kt : null,
    gs_kt: Number.isFinite(value.gs_kt) && value.gs_kt >= 0 ? value.gs_kt : null,
    airspeed_profile: normalizeAirspeedProfile(value.airspeed_profile),
    ias_acceleration_ktps: Number.isFinite(value.ias_acceleration_ktps) ? value.ias_acceleration_ktps : null,
  };
}

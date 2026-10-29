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
  const valid = object => Object.fromEntries(Object.entries(object).filter(([, v]) => Number.isFinite(v) && v > 0));
  const keys = ['vs0_kt', 'vs1_kt', 'vfe_kt', 'vno_kt', 'vne_kt', 'redline_kt'];
  const thresholds = valid(Object.fromEntries(keys.map(key => [key, value[key]])));
  const refKeys = ['vr_kt', 'vx_kt', 'vy_kt', 'vg_kt'];
  const references = valid(Object.fromEntries(refKeys.map(key => [key, value.references?.[key]])));
  if (!Object.keys(thresholds).length && !Object.keys(references).length) return null;
  return { ...thresholds, ...(Object.keys(references).length ? { references } : {}) };
}

export function normalizeOptionalIndications(value) {
  return {
    heading_deg: validHeading(value.heading_deg),
    selected_heading_deg: validHeading(value.selected_heading_deg),
    altitude_ft: Number.isFinite(value.altitude_ft) ? value.altitude_ft : null,
    selected_altitude_ft: Number.isFinite(value.selected_altitude_ft) ? value.selected_altitude_ft : null,
    vertical_speed_fpm: Number.isFinite(value.vertical_speed_fpm) ? value.vertical_speed_fpm : null,
    baro_text: typeof value.baro_text === 'string' ? value.baro_text : null,
    ...(Object.hasOwn(value, 'baro_setting') ? { baro_setting: normalizeBaro(value.baro_setting) } : {}),
    slip_skid: Number.isFinite(value.slip_skid) && Math.abs(value.slip_skid) <= 1 ? value.slip_skid : null,
    turn_rate_dps: Number.isFinite(value.turn_rate_dps) ? value.turn_rate_dps : null,
    ias_kt: Number.isFinite(value.ias_kt) && value.ias_kt >= 0 ? value.ias_kt : null,
    gs_kt: Number.isFinite(value.gs_kt) && value.gs_kt >= 0 ? value.gs_kt : null,
    tas_kt: Number.isFinite(value.tas_kt) && value.tas_kt >= 0 ? value.tas_kt : null,
    airspeed_profile: normalizeAirspeedProfile(value.airspeed_profile),
    ias_acceleration_ktps: Number.isFinite(value.ias_acceleration_ktps) ? value.ias_acceleration_ktps : null,
  };
}

function normalizeBaro(setting) {
  return setting && typeof setting === 'object' && !Array.isArray(setting) &&
    Number.isFinite(setting.pressure_hpa) && setting.pressure_hpa > 0 && typeof setting.standard === 'boolean'
    ? { pressure_hpa: setting.pressure_hpa, standard: setting.standard } : null;
}

function validHeading(value) {
  return Number.isFinite(value) && value >= 0 && value < 360 ? value : null;
}

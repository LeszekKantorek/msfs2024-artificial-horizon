import { svgElement, setAttributes } from '../helpers.js';

export function baroReading(value) {
  return typeof value === 'string' ? { state: 'available', text: value } : { state: 'unavailable', text: 'X' };
}

export function numericBaroReading(setting, unit) {
  if (!setting || !Number.isFinite(setting.pressure_hpa) || setting.pressure_hpa <= 0 ||
      typeof setting.standard !== 'boolean') return { state: 'unavailable', text: 'X' };
  if (setting.standard) return { state: 'available', text: 'STD' };
  const converted = unit === 'hPa' ? setting.pressure_hpa : setting.pressure_hpa / 33.8638866667;
  if (!Number.isFinite(converted) || converted >= 1e21) return { state: 'unavailable', text: 'X' };
  return { state: 'available', text: unit === 'hPa' ? `${Math.round(converted)} hPa` : `${converted.toFixed(2)} inHg` };
}

export function createBaro(parent, redraw = () => {}) {
  const group = svgElement('g', { id: 'barometric-setting', 'data-state': 'unavailable' });
  const background = svgElement('rect', { class: 'barometric-background' });
  const border = svgElement('rect', { class: 'barometric-border' });
  const value = svgElement('text', { id: 'barometric-value', class: 'barometric-value' });
  group.append(background, border, value); parent.append(group);
  let unit = 'inHg';
  let enabled = false;
  let availableWidth = 0;
  try {
    const saved = localStorage.getItem('efi.baro.unit');
    if (saved === 'inHg' || saved === 'hPa') unit = saved;
  } catch { /* The in-memory preference still works. */ }
  group.addEventListener('click', () => {
    if (!enabled) return;
    unit = unit === 'inHg' ? 'hPa' : 'inHg';
    try { localStorage.setItem('efi.baro.unit', unit); } catch { /* Storage is optional. */ }
    redraw();
  });
  return {
    resize({ x, y, width }) {
      availableWidth = width;
      group.setAttribute('transform', `translate(${x} ${y})`);
      setAttributes(background, { width, height: 32 });
      setAttributes(border, { x: 0.5, y: 0.5, width: width - 1, height: 31 });
      setAttributes(value, { x: width / 2, y: 16 });
    },
    invalidate() { enabled = false; group.style.cursor = ''; },
    render(setting, numeric, present = false) {
      let reading = present ? numericBaroReading(numeric, unit) : baroReading(setting);
      value.textContent = reading.text;
      if (present && reading.state === 'available' && availableWidth > 0 && value.getBBox().width > availableWidth - 2) {
        reading = { state: 'unavailable', text: 'X' };
      }
      enabled = present && numeric && Number.isFinite(numeric.pressure_hpa) && numeric.pressure_hpa > 0 &&
        typeof numeric.standard === 'boolean';
      group.style.cursor = enabled ? 'pointer' : '';
      group.dataset.state = reading.state;
      value.textContent = reading.text;
      group.setAttribute('aria-label', `Barometric setting ${reading.state === 'available' ? reading.text : reading.state}${enabled ? ', click or tap to switch units' : ''}`);
    },
  };
}

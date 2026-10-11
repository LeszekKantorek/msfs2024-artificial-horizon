import { svgElement, setAttributes } from '../helpers.js';
import { MAX_SPEED, tapeOffset } from './helpers.js';

// Compact fallback text preserves the configured value in the accessible label.
export function referenceReadout(value) {
  if (value > MAX_SPEED) return 'OVR';
  if (value < 0.1) return '<0.1';
  return String(Math.round(value * 10) / 10);
}

export function placeReferenceLabels(ias, values, tape) {
  const { centerY: center, clipTopY: top, clipBottomY: bottom } = tape;
  const positions = values.map(value => {
    const position = center + tapeOffset(ias, value);
    return { position, labelPosition: position, visible: position >= top + 8 && position <= bottom - 8 };
  });
  const visible = positions.filter(p => p.visible).sort((a, b) => a.position - b.position);
  // Pack colliding labels, then move the packed set back inside the clip if needed.
  for (let i = 1; i < visible.length; i++)
    visible[i].labelPosition = Math.max(visible[i].position, visible[i - 1].labelPosition + 16);
  if (visible.length && visible.at(-1).labelPosition > bottom - 8) {
    visible.at(-1).labelPosition = bottom - 8;
    for (let i = visible.length - 2; i >= 0; i--)
      visible[i].labelPosition = Math.min(visible[i].labelPosition, visible[i + 1].labelPosition - 16);
  }
  return positions;
}

export function createReferences(tapeHost, bottomHost) {
  const group = svgElement('g', { id: 'airspeed-references' });
  const records = [
    { key: 'vr_kt', symbol: 'R', accessibleName: 'VR' },
    { key: 'vx_kt', symbol: 'X', accessibleName: 'VX' },
    { key: 'vy_kt', symbol: 'Y', accessibleName: 'VY' },
    { key: 'vg_kt', symbol: 'G', accessibleName: 'VG' },
  ].map(reference => ({ ...reference,
    leader: svgElement('path', { class: 'airspeed-reference-leader', 'data-reference': reference.symbol }),
    marker: svgElement('text', { 'data-reference': reference.symbol, class: 'airspeed-reference' }, reference.symbol),
    fallbackLabel: svgElement('text', { class: 'airspeed-reference-value', 'data-reference': reference.symbol }),
  }));
  group.append(...records.map(reference => reference.leader), ...records.map(reference => reference.marker));
  tapeHost.append(group);
  const bottomGroup = svgElement('g', { id: 'airspeed-references-bottom', 'data-visible': 'false' });
  bottomGroup.append(...records.map(reference => reference.fallbackLabel));
  bottomHost.append(bottomGroup);
  let geometry;
  return {
    resize(tape) {
      geometry = tape;
      records.forEach((reference, index) => {
        reference.marker.setAttribute('x', tape.width - 4);
        setAttributes(reference.fallbackLabel, { x: tape.width / 2, y: tape.fallbackFirstLabelY + index * 16 });
      });
    },
    render(ias, profile, showBottomReferences) {
      group.style.display = profile && ias.state === 'available' ? '' : 'none';
      bottomGroup.dataset.visible = String(showBottomReferences);
      bottomGroup.style.display = showBottomReferences ? '' : 'none';
      if (!profile) return;
      const values = records.map(reference => profile.references[reference.key]);
      const positions = placeReferenceLabels(ias.value, values, geometry);
      records.forEach((reference, index) => {
        const value = values[index];
        const { marker, leader, fallbackLabel, symbol, accessibleName } = reference;
        fallbackLabel.textContent = `${symbol} ${referenceReadout(value)}`;
        const { position, labelPosition, visible } = positions[index];
        marker.style.display = ias.state === 'available' && visible ? '' : 'none';
        marker.setAttribute('y', Number.isFinite(labelPosition) ? labelPosition : 0);
        const displaced = ias.state === 'available' && visible && labelPosition !== position;
        leader.setAttribute('d', displaced ? `M ${geometry.width - 7} ${position} V ${labelPosition} H ${geometry.width - 5}` : '');
        marker.setAttribute('aria-label', `${accessibleName} ${value} knots`);
      });
      bottomGroup.setAttribute('aria-label', `Configured reference speeds in knots: ${records.map(reference => `${reference.accessibleName} ${profile.references[reference.key]}`).join(', ')}`);
    },
  };
}

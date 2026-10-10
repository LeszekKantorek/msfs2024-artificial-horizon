const AIRCRAFT_INNER_OFFSET = 25;
const AIRCRAFT_CHEVRON = 12;

export function createAircraft(group) {
  const aircraft = group.querySelector('.aircraft');
  const outline = group.querySelector('.aircraft-outline');
  return {
    resize(width) {
      const half = width / 2;
      const inner = AIRCRAFT_INNER_OFFSET, chevron = AIRCRAFT_CHEVRON;
      const shape = `M ${-half} 0 H ${-inner} l ${chevron} ${chevron} M ${inner - chevron} ${chevron} l ${chevron} -${chevron} H ${half} M -5 0 h 10`;
      aircraft.setAttribute('d', shape);
      outline.setAttribute('d', shape);
    },
  };
}

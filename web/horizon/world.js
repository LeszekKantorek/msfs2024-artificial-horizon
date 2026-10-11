export function createWorld(rotation) {
  const pitch = rotation.querySelector('#world-pitch');
  const sky = pitch.querySelector('.sky');
  const ground = pitch.querySelector('.ground');
  const horizon = pitch.querySelector('.horizon-line');
  return {
    resize({ width, height, pitchScale: pitchPixelsPerDegree }) {
      const cx = 0, cy = 0;
      // World extent covers every rotation and pitch displacement in the contract.
      const extent = Math.hypot(width, height) + 90 * pitchPixelsPerDegree;
      sky.setAttribute('d', `M ${cx - extent} ${cy - extent} H ${cx + extent} V ${cy} H ${cx - extent} Z`);
      ground.setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent} V ${cy + extent} H ${cx - extent} Z`);
      horizon.setAttribute('d', `M ${cx - extent} ${cy} H ${cx + extent}`);
    },
    render(transforms) {
      rotation.setAttribute('transform', transforms.rotation);
      pitch.setAttribute('transform', transforms.translation);
    },
  };
}

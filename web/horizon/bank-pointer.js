export function createBankPointer(origin) {
  const pointer = origin.querySelector('#bank-pointer');
  return {
    resize(layout) {
      origin.setAttribute('transform', `translate(${layout.cx} ${layout.cy})`);
      pointer.setAttribute('d', `M 0 ${-layout.bankRadius + 2} l -5 9 h 10 Z`);
    },
    render(transforms) {
      pointer.setAttribute('transform', transforms.rotation);
    },
  };
}

export function svgElement(name, attributes, text) {
  const element = document.createElementNS('http://www.w3.org/2000/svg', name);
  setAttributes(element, attributes);
  if (text !== undefined) element.textContent = text;
  return element;
}

export function setAttributes(element, values) {
  for (const [key, value] of Object.entries(values)) element.setAttribute(key, value);
}

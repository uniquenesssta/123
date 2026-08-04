export function isVersionAtLeast(actual, minimum) {
  const parse = (value) => String(value).split(".").map((part) => Number.parseInt(part, 10));
  const left = parse(actual);
  const right = parse(minimum);
  const length = Math.max(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    const current = Number.isFinite(left[index]) ? left[index] : 0;
    const required = Number.isFinite(right[index]) ? right[index] : 0;
    if (current !== required) return current > required;
  }
  return true;
}

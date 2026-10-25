/** Existing preference namespace; each application owner still owns its key and value policy. */
export function preferenceKey(name: string): string {
  if (!/^[a-z][a-z0-9-]*$/.test(name)) throw new TypeError("Invalid preference key");
  return `football-model-platform.${name}`;
}

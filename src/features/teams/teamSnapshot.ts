/** Detached owned responses: callers cannot mutate a different request's published state. */
export function teamSnapshot<T>(value: T): T {
  const result = structuredClone(value);
  const freeze = (item: unknown): void => {
    if (!item || typeof item !== "object" || Object.isFrozen(item)) return;
    for (const child of Object.values(item)) freeze(child);
    Object.freeze(item);
  };
  freeze(result); return result;
}

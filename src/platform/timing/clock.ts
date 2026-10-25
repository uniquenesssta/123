/** Wall time for timestamps; monotonic time for durations. Both are milliseconds. */
export const wallTime = (): number => Date.now();
export const monotonicTime = (): number => performance.now();

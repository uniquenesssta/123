/** Model-specific coverage requires a completed request with its own session, trace and input family. */
export function completedOperationsForGroup(entries, group, completedOperations) {
  if (!group.required_model_family) return completedOperations;
  const requests = new Map();
  const keyOf = (entry) => typeof entry.session_id === "string" && entry.session_id
    && typeof entry.trace_id === "string" && entry.trace_id
    ? JSON.stringify([entry.session_id, entry.trace_id, entry.details?.operation]) : null;
  for (const entry of entries) {
    const key = keyOf(entry);
    if (key === null) continue;
    const state = requests.get(key) ?? { families: [], completed: false, failed: false };
    if (entry.event === "operation_started") state.families.push(entry.details?.context?.args?.command?.model_family);
    if (entry.event === "operation_completed") state.completed = true;
    if (entry.event === "operation_failed") state.failed = true;
    requests.set(key, state);
  }
  return new Set([...requests].filter(([, state]) => state.completed && !state.failed
    && state.families.length === 1 && state.families[0] === group.required_model_family)
    .map(([key]) => JSON.parse(key)[2]));
}

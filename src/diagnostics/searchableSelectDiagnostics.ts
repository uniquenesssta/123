import { recordFrontendDiagnostic } from "../api/client";

const SEARCHABLE_SELECT_DIAGNOSTIC_EVENT = "football:searchable-select-diagnostic";
let bound = false;

interface SearchableSelectDiagnosticDetail {
  readonly event: string;
  readonly context: Record<string, unknown>;
}

function isDiagnosticDetail(value: unknown): value is SearchableSelectDiagnosticDetail {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<SearchableSelectDiagnosticDetail>;
  return typeof candidate.event === "string"
    && candidate.event.length > 0
    && Boolean(candidate.context)
    && typeof candidate.context === "object";
}

function handleSearchableSelectDiagnostic(event: Event): void {
  if (!(event instanceof CustomEvent) || !isDiagnosticDetail(event.detail)) return;
  recordFrontendDiagnostic(
    `searchable_select_${event.detail.event}`,
    event.detail.context,
  );
}

export function bindSearchableSelectDiagnostics(signal?: AbortSignal): void {
  if (bound || signal?.aborted) return;
  bound = true;
  document.addEventListener(
    SEARCHABLE_SELECT_DIAGNOSTIC_EVENT,
    handleSearchableSelectDiagnostic,
    signal ? { signal } : undefined,
  );
  signal?.addEventListener("abort", () => {
    bound = false;
  }, { once: true });
}

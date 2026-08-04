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

export function bindSearchableSelectDiagnostics(): void {
  if (bound) return;
  bound = true;
  document.addEventListener(SEARCHABLE_SELECT_DIAGNOSTIC_EVENT, (event) => {
    if (!(event instanceof CustomEvent) || !isDiagnosticDetail(event.detail)) return;
    recordFrontendDiagnostic(
      `searchable_select_${event.detail.event}`,
      event.detail.context,
    );
  });
}

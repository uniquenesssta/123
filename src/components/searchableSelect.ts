type SearchableOption = {
  readonly option: HTMLOptionElement;
  readonly label: string;
  readonly searchText: string;
  readonly originalIndex: number;
};

type SearchableSelectController = {
  readonly select: HTMLSelectElement;
  readonly root: HTMLDivElement;
  readonly input: HTMLInputElement;
  readonly toggle: HTMLButtonElement;
  readonly listbox: HTMLDivElement;
  readonly empty: HTMLDivElement;
  activeIndex: number;
  filtered: SearchableOption[];
  open: boolean;
  querying: boolean;
  composing: boolean;
  queryValue: string;
  restoringDraft: boolean;
  diagnosticSessionId: string | null;
  lastDiagnosticSignature: string;
  lastSelectedValue: string;
  lastSelectedLabel: string;
};

const controllers = new WeakMap<HTMLSelectElement, SearchableSelectController>();
let activeController: SearchableSelectController | null = null;
let globalListenersBound = false;
let globalObserver: MutationObserver | null = null;
let generatedSelectId = 0;
const activeQueryDrafts = new Map<string, string>();
const activeDiagnosticSessions = new Map<string, string>();
let resumableSelectId: string | null = null;
let diagnosticSequence = 0;
const SEARCHABLE_SELECT_DIAGNOSTIC_EVENT = "football:searchable-select-diagnostic";

function normalizeSearchText(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLocaleLowerCase("zh-CN")
    .replace(/[·•・_/\\|()[\]{}:：,，.。'"`~!！?？+\-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function compactSearchText(value: string): string {
  return normalizeSearchText(value).replace(/\s+/g, "");
}

function isSubsequence(query: string, candidate: string): boolean {
  if (!query) return true;
  let index = 0;
  for (const character of candidate) {
    if (character === query[index]) index += 1;
    if (index === query.length) return true;
  }
  return false;
}

function fuzzyScore(query: string, candidate: string): number | null {
  if (!query) return 0;
  const normalizedQuery = normalizeSearchText(query);
  const normalizedCandidate = normalizeSearchText(candidate);
  if (!normalizedQuery) return 0;
  if (normalizedCandidate === normalizedQuery) return 0;
  if (normalizedCandidate.startsWith(normalizedQuery)) return 1;
  if (normalizedCandidate.split(" ").some((token) => token.startsWith(normalizedQuery))) return 2;
  if (normalizedCandidate.includes(normalizedQuery)) return 3;
  const compactQuery = compactSearchText(normalizedQuery);
  const compactCandidate = compactSearchText(normalizedCandidate);
  if (compactCandidate.includes(compactQuery)) return 4;
  if (compactQuery.length >= 2 && isSubsequence(compactQuery, compactCandidate)) return 5;
  return null;
}

function selectedLabel(select: HTMLSelectElement): string {
  return select.selectedOptions[0]?.textContent?.trim() ?? "";
}

function clipDiagnosticText(value: string, maximum = 240): string {
  const characters = [...value];
  if (characters.length <= maximum) return value;
  return `${characters.slice(0, maximum).join("")}…`;
}

function fieldLabel(select: HTMLSelectElement): string {
  const explicit = select.getAttribute("aria-label")?.trim();
  if (explicit) return explicit;
  const label = select.closest("label");
  const direct = label?.querySelector<HTMLElement>(":scope > span")?.textContent?.trim();
  if (direct) return direct;
  return select.name || select.id;
}

function describeElement(element: Element | null): string | null {
  if (!element) return null;
  const tag = element.tagName.toLocaleLowerCase("en-US");
  const id = element.id ? `#${element.id}` : "";
  const classes = [...element.classList].slice(0, 3).map((name) => `.${name}`).join("");
  return `${tag}${id}${classes}`;
}

function ensureDiagnosticSession(controller: SearchableSelectController): string {
  const existing = controller.diagnosticSessionId
    ?? activeDiagnosticSessions.get(controller.select.id);
  if (existing) {
    controller.diagnosticSessionId = existing;
    return existing;
  }
  diagnosticSequence += 1;
  const sessionId = `${controller.select.id}-${Date.now()}-${diagnosticSequence}`;
  controller.diagnosticSessionId = sessionId;
  activeDiagnosticSessions.set(controller.select.id, sessionId);
  return sessionId;
}

function diagnosticContext(
  controller: SearchableSelectController,
  extra: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    diagnostic_session_id: controller.diagnosticSessionId
      ?? activeDiagnosticSessions.get(controller.select.id)
      ?? null,
    selector_id: controller.select.id,
    selector_name: controller.select.name || null,
    field_label: fieldLabel(controller.select),
    page_path: `${location.pathname}${location.hash}`,
    selected_value: clipDiagnosticText(controller.select.value),
    selected_label: clipDiagnosticText(selectedLabel(controller.select)),
    input_value: clipDiagnosticText(controller.input.value),
    query_value: clipDiagnosticText(controller.queryValue),
    open: controller.open,
    querying: controller.querying,
    composing: controller.composing,
    input_focused: document.activeElement === controller.input,
    root_connected: controller.root.isConnected,
    select_connected: controller.select.isConnected,
    option_count: controller.select.options.length,
    active_element: describeElement(document.activeElement),
    ...extra,
  };
}

function emitDiagnostic(
  controller: SearchableSelectController,
  event: string,
  extra: Record<string, unknown> = {},
): void {
  document.dispatchEvent(new CustomEvent(SEARCHABLE_SELECT_DIAGNOSTIC_EVENT, {
    detail: {
      event,
      context: diagnosticContext(controller, extra),
    },
  }));
}

function scheduleQueryConsistencyCheck(
  controller: SearchableSelectController,
  trigger: string,
): void {
  queueMicrotask(() => {
    if (!controller.querying || controller.restoringDraft || !controller.root.isConnected) return;
    const actual = controller.input.value;
    const expected = controller.queryValue;
    if (actual === expected) {
      controller.lastDiagnosticSignature = "";
      return;
    }
    const signature = `${trigger}\u0000${expected}\u0000${actual}`;
    if (signature === controller.lastDiagnosticSignature) return;
    controller.lastDiagnosticSignature = signature;
    emitDiagnostic(controller, "query_value_diverged", {
      severity: "warning",
      trigger,
      expected_query: clipDiagnosticText(expected),
      actual_input: clipDiagnosticText(actual),
    });
  });
}

function searchableOptions(select: HTMLSelectElement): SearchableOption[] {
  return Array.from(select.options)
    .map((option, originalIndex) => ({
      option,
      label: option.textContent?.trim() ?? "",
      searchText: `${option.textContent ?? ""} ${option.value} ${option.dataset.search ?? ""}`,
      originalIndex,
    }))
    .filter(({ option }) => !option.hidden && !option.disabled);
}

function captureDetachedActiveController(): void {
  if (!activeController || activeController.root.isConnected) return;
  const controller = activeController;
  const query = controller.input.value;
  if (controller.querying || document.activeElement === controller.input) {
    ensureDiagnosticSession(controller);
    activeQueryDrafts.set(controller.select.id, query);
    activeDiagnosticSessions.set(controller.select.id, controller.diagnosticSessionId ?? "");
    resumableSelectId = controller.select.id;
    emitDiagnostic(controller, "dom_detached_during_query", {
      severity: controller.composing ? "warning" : "info",
      draft_query: clipDiagnosticText(query),
      expected_query: clipDiagnosticText(controller.queryValue),
      composition_interrupted: controller.composing,
    });
  }
  activeController = null;
}

function setExpanded(
  controller: SearchableSelectController,
  expanded: boolean,
  restoreSelection = true,
  reason = "programmatic",
): void {
  if (controller.select.disabled) expanded = false;
  if (expanded && activeController && activeController !== controller) {
    setExpanded(activeController, false, true, "switch_controller");
  }
  controller.open = expanded;
  controller.root.classList.toggle("open", expanded);
  controller.input.setAttribute("aria-expanded", String(expanded));
  controller.toggle.setAttribute("aria-expanded", String(expanded));
  controller.listbox.hidden = !expanded;
  if (expanded) {
    activeController = controller;
    renderOptions(controller, controller.querying ? controller.queryValue : "");
  } else if (activeController === controller) {
    activeController = null;
    controller.activeIndex = -1;
    const beforeInput = controller.input.value;
    const beforeQuery = controller.queryValue;
    const selected = selectedLabel(controller.select);
    if (
      restoreSelection
      && beforeQuery.length > 0
      && beforeInput !== selected
      && reason !== "selection_commit"
      && reason !== "escape_cancel"
    ) {
      emitDiagnostic(controller, "query_overwritten_by_selection", {
        severity: "warning",
        reason,
        before_input: clipDiagnosticText(beforeInput),
        before_query: clipDiagnosticText(beforeQuery),
        replacement_label: clipDiagnosticText(selected),
      });
    }
    controller.querying = false;
    controller.composing = false;
    controller.queryValue = "";
    if (controller.root.isConnected) {
      activeQueryDrafts.delete(controller.select.id);
      activeDiagnosticSessions.delete(controller.select.id);
      if (resumableSelectId === controller.select.id) resumableSelectId = null;
    }
    if (restoreSelection) controller.input.value = selected;
    controller.diagnosticSessionId = null;
    controller.lastDiagnosticSignature = "";
  }
}

function focusActiveOption(controller: SearchableSelectController): void {
  const items = Array.from(controller.listbox.querySelectorAll<HTMLButtonElement>(".searchable-select-option"));
  items.forEach((item, index) => item.classList.toggle("active", index === controller.activeIndex));
  const active = items[controller.activeIndex];
  if (active) {
    controller.input.setAttribute("aria-activedescendant", active.id);
    active.scrollIntoView({ block: "nearest" });
  } else {
    controller.input.removeAttribute("aria-activedescendant");
  }
}

function chooseOption(controller: SearchableSelectController, item: SearchableOption): void {
  activeQueryDrafts.delete(controller.select.id);
  activeDiagnosticSessions.delete(controller.select.id);
  if (resumableSelectId === controller.select.id) resumableSelectId = null;
  controller.querying = false;
  controller.composing = false;
  controller.queryValue = "";
  controller.select.value = item.option.value;
  controller.input.value = item.label;
  controller.select.dispatchEvent(new Event("change", { bubbles: true }));
  setExpanded(controller, false, true, "selection_commit");
  controller.input.focus({ preventScroll: true });
}

function renderOptions(controller: SearchableSelectController, query: string): void {
  const scored = searchableOptions(controller.select)
    .map((item) => ({ item, score: fuzzyScore(query, item.searchText) }))
    .filter((entry): entry is { item: SearchableOption; score: number } => entry.score !== null)
    .sort((left, right) => left.score - right.score || left.item.originalIndex - right.item.originalIndex)
    .slice(0, 120)
    .map((entry) => entry.item);

  controller.filtered = scored;
  controller.listbox.replaceChildren();
  controller.empty.hidden = scored.length > 0;
  if (scored.length === 0) {
    controller.listbox.append(controller.empty);
    controller.activeIndex = -1;
    return;
  }

  const selectedValue = controller.select.value;
  for (const [index, item] of scored.entries()) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "searchable-select-option";
    button.id = `${controller.select.id}-search-option-${index}`;
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", String(item.option.value === selectedValue));
    button.dataset.value = item.option.value;
    const label = document.createElement("span");
    label.textContent = item.label;
    const marker = document.createElement("b");
    marker.textContent = item.option.value === selectedValue ? "✓" : "";
    button.append(label, marker);
    button.addEventListener("mousedown", (event) => event.preventDefault());
    button.addEventListener("click", () => chooseOption(controller, item));
    controller.listbox.append(button);
  }

  controller.activeIndex = Math.max(0, scored.findIndex((item) => item.option.value === selectedValue));
  focusActiveOption(controller);
}

function syncController(controller: SearchableSelectController): void {
  const disabled = controller.select.disabled;
  controller.root.classList.toggle("disabled", disabled);
  controller.input.disabled = disabled;
  controller.toggle.disabled = disabled;
  controller.input.placeholder = controller.select.dataset.searchPlaceholder
    ?? controller.select.options[0]?.textContent?.trim()
    ?? "输入关键词筛选";
  const inputFocused = document.activeElement === controller.input;
  const hasResumableDraft = resumableSelectId === controller.select.id
    && activeQueryDrafts.has(controller.select.id);
  const preserveDraft = inputFocused
    || hasResumableDraft
    || controller.composing
    || (controller.open && controller.querying);
  const currentSelectedValue = controller.select.value;
  const currentSelectedLabel = selectedLabel(controller.select);
  if (
    controller.querying
    && controller.queryValue.length > 0
    && (currentSelectedValue !== controller.lastSelectedValue
      || currentSelectedLabel !== controller.lastSelectedLabel)
  ) {
    emitDiagnostic(controller, "selection_changed_during_query", {
      severity: "info",
      previous_selected_value: clipDiagnosticText(controller.lastSelectedValue),
      previous_selected_label: clipDiagnosticText(controller.lastSelectedLabel),
      next_selected_value: clipDiagnosticText(currentSelectedValue),
      next_selected_label: clipDiagnosticText(currentSelectedLabel),
      draft_preserved: preserveDraft,
    });
  }
  if (!preserveDraft) {
    const previousInput = controller.input.value;
    if (
      previousInput.length > 0
      && previousInput !== currentSelectedLabel
      && (controller.querying || activeQueryDrafts.has(controller.select.id))
    ) {
      emitDiagnostic(controller, "query_overwritten_by_sync", {
        severity: "warning",
        before_input: clipDiagnosticText(previousInput),
        replacement_label: clipDiagnosticText(currentSelectedLabel),
        input_focused: inputFocused,
        resumable_draft_present: hasResumableDraft,
      });
    }
    controller.input.value = currentSelectedLabel;
    controller.queryValue = "";
  } else {
    scheduleQueryConsistencyCheck(controller, "sync_controller");
  }
  controller.lastSelectedValue = currentSelectedValue;
  controller.lastSelectedLabel = currentSelectedLabel;
  if (controller.open) {
    renderOptions(controller, controller.querying ? controller.queryValue : "");
  }
}

function ensureSelectId(select: HTMLSelectElement): void {
  if (select.id) return;
  generatedSelectId += 1;
  select.id = `searchable-select-${generatedSelectId}`;
}

function applyControllerVariant(controller: SearchableSelectController): void {
  const compact = Boolean(controller.select.closest(
    ".balanced-lineup-row, .lineup-builder-row, .field.compact, .compact-form, .table-row-actions",
  ));
  const inline = controller.select.matches(
    ".balanced-lineup-role, .balanced-lineup-position, [data-lineup-field]",
  );
  controller.root.classList.toggle("compact", compact || inline);
  controller.root.classList.toggle("inline", inline);
}

function isEligibleSelect(select: HTMLSelectElement): boolean {
  if (select.multiple || select.size > 1) return false;
  if (select.matches("[data-native-select], [data-searchable-select='off']")) return false;
  return !select.classList.contains("searchable-select-native");
}

function eligibleSelects(root: ParentNode): HTMLSelectElement[] {
  const found: HTMLSelectElement[] = [];
  if (root instanceof HTMLSelectElement && isEligibleSelect(root)) found.push(root);
  root.querySelectorAll<HTMLSelectElement>("select:not([multiple])").forEach((select) => {
    if (isEligibleSelect(select)) found.push(select);
  });
  return found;
}

function createController(select: HTMLSelectElement): SearchableSelectController {
  ensureSelectId(select);
  const root = document.createElement("div");
  root.className = "searchable-select";

  const control = document.createElement("div");
  control.className = "searchable-select-control";

  const input = document.createElement("input");
  input.type = "text";
  input.autocomplete = "off";
  input.spellcheck = false;
  input.className = "searchable-select-input";
  input.setAttribute("role", "combobox");
  input.setAttribute("aria-autocomplete", "list");
  input.setAttribute("aria-expanded", "false");
  input.setAttribute("aria-controls", `${select.id}-search-listbox`);

  const toggle = document.createElement("button");
  toggle.type = "button";
  toggle.className = "searchable-select-toggle";
  toggle.setAttribute("aria-label", "展开可搜索选项");
  toggle.setAttribute("aria-expanded", "false");
  toggle.textContent = "⌄";

  const listbox = document.createElement("div");
  listbox.id = `${select.id}-search-listbox`;
  listbox.className = "searchable-select-listbox";
  listbox.setAttribute("role", "listbox");
  listbox.hidden = true;

  const empty = document.createElement("div");
  empty.className = "searchable-select-empty";
  empty.textContent = "没有匹配项";
  empty.hidden = true;

  select.parentNode?.insertBefore(root, select);
  root.append(control, select, listbox);
  control.append(input, toggle);
  select.classList.add("searchable-select-native");

  const controller: SearchableSelectController = {
    select,
    root,
    input,
    toggle,
    listbox,
    empty,
    activeIndex: -1,
    filtered: [],
    open: false,
    querying: false,
    composing: false,
    queryValue: "",
    restoringDraft: false,
    diagnosticSessionId: null,
    lastDiagnosticSignature: "",
    lastSelectedValue: select.value,
    lastSelectedLabel: selectedLabel(select),
  };

  input.addEventListener("focus", () => {
    const selected = selectedLabel(select);
    const startFreshQuery = !controller.restoringDraft
      && (!controller.querying || input.value === selected);
    controller.querying = true;
    resumableSelectId = select.id;
    if (!controller.restoringDraft) {
      if (startFreshQuery) {
        ensureDiagnosticSession(controller);
        emitDiagnostic(controller, "query_session_started", {
          severity: "info",
          trigger: "focus",
          prior_display_value: clipDiagnosticText(input.value),
        });
        input.value = "";
      } else {
        ensureDiagnosticSession(controller);
      }
      controller.queryValue = input.value;
      activeQueryDrafts.set(select.id, controller.queryValue);
      activeDiagnosticSessions.set(select.id, controller.diagnosticSessionId ?? "");
      requestAnimationFrame(() => {
        if (document.activeElement !== input) return;
        const caret = input.value.length;
        input.setSelectionRange(caret, caret);
        scheduleQueryConsistencyCheck(controller, "focus_caret_restore");
      });
    }
    setExpanded(controller, true, false, "focus_open");
  });
  input.addEventListener("beforeinput", (event) => {
    if (!event.inputType.startsWith("insert")) return;
    if (controller.queryValue || input.value !== selectedLabel(select)) return;
    input.value = "";
    input.setSelectionRange(0, 0);
  });
  input.addEventListener("click", () => {
    controller.querying = true;
    ensureDiagnosticSession(controller);
    setExpanded(controller, true, false, "input_click");
  });
  input.addEventListener("compositionstart", () => {
    const sessionWasMissing = !controller.diagnosticSessionId
      && !activeDiagnosticSessions.has(select.id);
    ensureDiagnosticSession(controller);
    if (sessionWasMissing) {
      emitDiagnostic(controller, "query_session_started", {
        severity: "info",
        trigger: "composition_start",
        prior_display_value: clipDiagnosticText(input.value),
      });
    }
    controller.composing = true;
    controller.querying = true;
    resumableSelectId = select.id;
    activeDiagnosticSessions.set(select.id, controller.diagnosticSessionId ?? "");
    setExpanded(controller, true, false, "composition_start");
  });
  input.addEventListener("compositionend", () => {
    controller.composing = false;
    controller.querying = true;
    controller.queryValue = input.value;
    activeQueryDrafts.set(select.id, controller.queryValue);
    resumableSelectId = select.id;
    renderOptions(controller, controller.queryValue);
    scheduleQueryConsistencyCheck(controller, "composition_end");
  });
  input.addEventListener("input", () => {
    const sessionWasMissing = !controller.diagnosticSessionId
      && !activeDiagnosticSessions.has(select.id);
    controller.querying = true;
    controller.queryValue = input.value;
    activeQueryDrafts.set(select.id, controller.queryValue);
    resumableSelectId = select.id;
    ensureDiagnosticSession(controller);
    if (sessionWasMissing) {
      emitDiagnostic(controller, "query_session_started", {
        severity: "info",
        trigger: "input_event",
        prior_display_value: clipDiagnosticText(input.value),
      });
    }
    activeDiagnosticSessions.set(select.id, controller.diagnosticSessionId ?? "");
    setExpanded(controller, true, false, "user_input");
    if (!controller.composing) renderOptions(controller, controller.queryValue);
    scheduleQueryConsistencyCheck(controller, "input_event");
  });
  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!controller.open) setExpanded(controller, true, false, "keyboard_open");
      if (controller.filtered.length === 0) return;
      const direction = event.key === "ArrowDown" ? 1 : -1;
      controller.activeIndex = (controller.activeIndex + direction + controller.filtered.length) % controller.filtered.length;
      focusActiveOption(controller);
      return;
    }
    if (event.key === "Enter") {
      if (controller.composing || !controller.open || controller.activeIndex < 0) return;
      event.preventDefault();
      const item = controller.filtered[controller.activeIndex];
      if (item) chooseOption(controller, item);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setExpanded(controller, false, true, "escape_cancel");
    }
  });
  input.addEventListener("blur", () => {
    const queryAtBlur = controller.queryValue;
    const inputAtBlur = input.value;
    const wasComposing = controller.composing;
    queueMicrotask(() => {
      const active = document.activeElement;
      if (active instanceof Node && controller.root.contains(active)) return;
      if (queryAtBlur.length > 0 || wasComposing) {
        emitDiagnostic(controller, "focus_lost_with_active_query", {
          severity: wasComposing ? "warning" : "info",
          query_at_blur: clipDiagnosticText(queryAtBlur),
          input_at_blur: clipDiagnosticText(inputAtBlur),
          composition_interrupted: wasComposing,
          next_active_element: describeElement(active),
        });
      }
      if (controller.open) setExpanded(controller, false, true, "focus_lost");
    });
  });
  toggle.addEventListener("mousedown", (event) => event.preventDefault());
  toggle.addEventListener("click", () => {
    if (controller.open) {
      setExpanded(controller, false, true, "toggle_close");
    } else {
      input.focus({ preventScroll: true });
      setExpanded(controller, true, false, "toggle_open");
    }
  });

  controllers.set(select, controller);
  select.dataset.searchableSelect = "enhanced";
  applyControllerVariant(controller);
  syncController(controller);
  const draft = activeQueryDrafts.get(select.id);
  if (draft !== undefined && resumableSelectId === select.id) {
    controller.diagnosticSessionId = activeDiagnosticSessions.get(select.id) ?? null;
    queueMicrotask(() => {
      if (!controller.root.isConnected) {
        ensureDiagnosticSession(controller);
        emitDiagnostic(controller, "query_restore_failed", {
          severity: "warning",
          reason: "replacement_controller_disconnected",
          expected_query: clipDiagnosticText(draft),
        });
        return;
      }
      ensureDiagnosticSession(controller);
      controller.restoringDraft = true;
      controller.input.value = draft;
      controller.queryValue = draft;
      controller.querying = true;
      setExpanded(controller, true, false, "dom_rebuild_restore");
      controller.input.focus({ preventScroll: true });
      controller.input.setSelectionRange(draft.length, draft.length);
      controller.restoringDraft = false;
      renderOptions(controller, draft);
      emitDiagnostic(controller, "query_restored_after_dom_rebuild", {
        severity: "info",
        restored_query: clipDiagnosticText(draft),
        caret_start: controller.input.selectionStart,
        caret_end: controller.input.selectionEnd,
      });
      requestAnimationFrame(() => {
        const inputMatches = controller.input.value === draft;
        const queryMatches = controller.queryValue === draft;
        const focusRestored = document.activeElement === controller.input;
        const caretRestored = controller.input.selectionStart === draft.length
          && controller.input.selectionEnd === draft.length;
        if (!inputMatches || !queryMatches || !focusRestored || !caretRestored) {
          emitDiagnostic(controller, "query_restore_failed", {
            severity: "warning",
            reason: "post_restore_verification",
            expected_query: clipDiagnosticText(draft),
            input_matches: inputMatches,
            query_matches: queryMatches,
            focus_restored: focusRestored,
            caret_restored: caretRestored,
            caret_start: controller.input.selectionStart,
            caret_end: controller.input.selectionEnd,
          });
        }
        scheduleQueryConsistencyCheck(controller, "post_dom_restore");
      });
    });
  }
  return controller;
}

function bindGlobalListeners(): void {
  if (globalListenersBound) return;
  globalListenersBound = true;
  document.addEventListener("pointerdown", (event) => {
    if (!activeController) return;
    const target = event.target;
    if (target instanceof Node && activeController.root.contains(target)) return;
    setExpanded(activeController, false, true, "outside_pointer");
  });
}

function bindGlobalObserver(): void {
  if (globalObserver || typeof MutationObserver === "undefined") return;
  globalObserver = new MutationObserver((mutations) => {
    captureDetachedActiveController();
    for (const mutation of mutations) {
      if (mutation.type === "attributes") {
        const target = mutation.target;
        const select = target instanceof HTMLSelectElement
          ? target
          : target instanceof HTMLOptionElement
            ? target.closest("select")
            : null;
        if (select instanceof HTMLSelectElement) {
          const controller = controllers.get(select);
          if (controller) syncController(controller);
        }
        continue;
      }
      if (mutation.target instanceof HTMLSelectElement) {
        const controller = controllers.get(mutation.target);
        if (controller) syncController(controller);
      }
      mutation.addedNodes.forEach((node) => {
        if (node instanceof Element) enhanceSearchableSelects(node);
      });
    }
  });
  globalObserver.observe(document.documentElement, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["disabled", "hidden", "selected", "value"],
  });
}

export function enhanceSearchableSelects(root: ParentNode = document): void {
  captureDetachedActiveController();
  bindGlobalListeners();
  bindGlobalObserver();
  eligibleSelects(root).forEach((select) => {
    const controller = controllers.get(select) ?? createController(select);
    applyControllerVariant(controller);
    syncController(controller);
  });
}

export function refreshSearchableSelects(root: ParentNode = document): void {
  captureDetachedActiveController();
  const selects = root instanceof HTMLSelectElement
    ? [root]
    : Array.from(root.querySelectorAll<HTMLSelectElement>("select"));
  selects.forEach((select) => {
    const controller = controllers.get(select);
    if (controller) {
      applyControllerVariant(controller);
      syncController(controller);
    } else if (isEligibleSelect(select)) {
      createController(select);
    }
  });
}

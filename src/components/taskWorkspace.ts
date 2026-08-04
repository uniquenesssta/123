import { escapeHtml } from "./format";

export type TaskTone = "neutral" | "accent" | "success" | "warning" | "danger";

export interface TaskStatusView {
  readonly label: string;
  readonly tone?: TaskTone;
}

export interface TaskPageHeaderOptions {
  readonly eyebrow: string;
  readonly title: string;
  readonly description: string;
  readonly status?: TaskStatusView;
  readonly actions?: string;
}

export interface TaskContextItem {
  readonly label: string;
  readonly value: string;
  readonly note?: string;
  readonly tone?: TaskTone;
  readonly actions?: string;
}

function toneClass(tone: TaskTone | undefined): string {
  return `tone-${tone ?? "neutral"}`;
}

export function taskStatusChip(status: TaskStatusView): string {
  return `<span class="task-status-chip ${toneClass(status.tone)}">${escapeHtml(status.label)}</span>`;
}

export function taskPageHeader(options: TaskPageHeaderOptions): string {
  const trailing = [options.status ? taskStatusChip(options.status) : "", options.actions ?? ""]
    .filter(Boolean)
    .join("");
  return `<header class="task-page-header"><div class="task-page-heading"><p class="eyebrow">${escapeHtml(options.eyebrow)}</p><h1>${escapeHtml(options.title)}</h1><p>${escapeHtml(options.description)}</p></div>${trailing ? `<div class="task-page-actions">${trailing}</div>` : ""}</header>`;
}

export function taskContextRibbon(items: readonly TaskContextItem[]): string {
  if (!items.length) return "";
  return `<section class="task-context-ribbon" aria-label="当前任务上下文">${items.map((item) => `<article class="task-context-item ${toneClass(item.tone)} "><div><span>${escapeHtml(item.label)}</span><strong>${escapeHtml(item.value)}</strong>${item.note ? `<small>${escapeHtml(item.note)}</small>` : ""}</div>${item.actions ? `<div class="task-context-actions">${item.actions}</div>` : ""}</article>`).join("")}</section>`;
}

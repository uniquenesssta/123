import { createApplication } from "./createApplication";
import type { ApplicationHandle } from "./applicationHandle";

function errorMessage(error: unknown): string {
  if (error instanceof AggregateError && error.errors.length > 0) {
    return errorMessage(error.errors[0]);
  }
  return error instanceof Error ? error.message : String(error);
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function renderStartupFailure(root: HTMLElement | null, error: unknown): void {
  if (!root) return;
  root.innerHTML = `<div class="fatal"><strong>平台启动失败</strong><p>${escapeHtml(errorMessage(error))}</p><small>完整错误已写入问题日志。</small></div>`;
}

export async function startApplication(): Promise<ApplicationHandle | null> {
  let handle: ApplicationHandle | null = null;
  try {
    handle = createApplication();
    await handle.start();
    return handle;
  } catch (error: unknown) {
    renderStartupFailure(handle?.root ?? document.querySelector<HTMLElement>("#app"), error);
    return null;
  }
}

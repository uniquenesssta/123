import { escapeHtml } from "../../../components/format";
export interface CodeBlockOptions { label: string; text: string; wrap?: boolean }
/** Escaped read-only text. Redaction and acquisition remain the host's responsibility. */
export function codeBlockView(o: CodeBlockOptions): string {
  if (!o.label.trim()) throw new Error("Code Block requires an accessible name");
  return `<pre class="ui-code-block" data-ui-display="code-block" data-wrap="${o.wrap !== false}" tabindex="0" role="region" aria-label="${escapeHtml(o.label)}"><code>${escapeHtml(o.text)}</code></pre>`;
}

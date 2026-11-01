import { escapeHtml } from '../../components/format';
import { buttonView } from '../../ui/primitives/button';
import { textInputView } from '../../ui/primitives/text-input';
import { numberInputView } from '../../ui/primitives/number-input';
import { inlineMessageView } from '../../ui/feedback/inline-message';
export const html = escapeHtml;
export const reviewNumber = (value: unknown, digits = 1) => typeof value === 'number' && Number.isFinite(value) ? value.toFixed(digits) : '—';
export const reviewPercent = (value: unknown) => typeof value === 'number' && Number.isFinite(value) ? new Intl.NumberFormat('zh-CN', { style: 'percent', maximumFractionDigits: 1 }).format(value) : '—';
export function reviewPanel(title: string, description: string, body: string, classes = ''): string { return `<section class="review-panel ${classes}"><header><h2>${html(title)}</h2><p>${html(description)}</p></header><div class="review-panel-body">${body}</div></section>`; }
export function reviewButton(label: string, action: string, disabled = false, primary = false, attributes: Record<string, string> = {}): string { return buttonView({ label, variant: primary ? 'primary' : 'secondary', disabled, attributes: { 'data-action': action, ...attributes } }); }
export function reviewText(label: string, key: string, value: string): string { return `<label class="review-field"><span>${html(label)}</span>${textInputView({ label, value, attributes: { id: 'review-' + key, 'data-review-fact': key } })}</label>`; }
export function reviewNumeric(label: string, key: string, value: string, max?: number, step: number | 'any' = 1, playerField: string | false = false): string { return `<label class="review-field"><span>${html(label)}</span>${numberInputView({ label, value, min: 0, max, step, attributes: playerField ? { id: `review-${playerField}-${key}`, 'data-field': key } : { id: 'review-' + key, 'data-review-fact': key } })}</label>`; }
export function reviewAlert(title: string, description: string, tone: 'error' | 'warning' | 'info' = 'warning'): string { return inlineMessageView({ title, message: description, tone: tone === 'error' ? 'danger' : tone, live: true }); }

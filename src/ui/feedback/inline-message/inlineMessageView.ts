import { alertView, type AlertOptions } from "../alert";
/** The caller keeps the field association and validation gate even when hidden. */
export function inlineMessageView(options: AlertOptions): string { return alertView(options); }

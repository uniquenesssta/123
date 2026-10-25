import { buttonView, type ButtonOptions } from "../../primitives/button";

export interface FormActionsOptions { cancel?: ButtonOptions; confirm: ButtonOptions; busy?: boolean; framed?: boolean }

/** Host owns requests. Busy projects both actions without starting work. */
export function formActionsView(o: FormActionsOptions): string {
  const busy = !!o.busy || !!o.confirm.loading;
  return `<div class="ui-form-actions" ${o.framed ? 'data-framed="true"' : ""}>${o.cancel ? buttonView({ ...o.cancel, variant: "secondary", loading: false, disabled: o.cancel.disabled || busy }) : ""}${buttonView({ ...o.confirm, disabled: o.confirm.disabled || busy, loading: busy })}</div>`;
}

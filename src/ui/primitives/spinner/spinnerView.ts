import small from "../../../assets/icons/spinner.svg?raw";
import medium from "../../../assets/controls/spinner-20.svg?raw";
import large from "../../../assets/controls/spinner-24.svg?raw";
import { primitiveName } from "../controlAttributes";

/** Exact exported geometry; semantic paints and rotation are supplied by CSS. */
export function spinnerView(size: 16 | 20 | 24 = 16, tone: "Default" | "On Accent" = "Default", label?: string): string {
  return `<span class="ui-spinner ui-icon-spinner ui-spinner-${size}" data-ui-primitive="spinner" data-tone="${tone}" ${label ? `role="status" aria-label="${primitiveName(label)}"` : 'aria-hidden="true"'}>${{ 16: small, 20: medium, 24: large }[size]}</span>`;
}

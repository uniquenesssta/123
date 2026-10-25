import { primitiveName } from "../controlAttributes";

/** Button/Icon's approved tooltip is the platform-native title, with an AT description. */
export function tooltipView(id: string, text: string): string {
  return `<span class="ui-tooltip-description" id="${primitiveName(id)}" role="tooltip">${primitiveName(text)}</span>`;
}

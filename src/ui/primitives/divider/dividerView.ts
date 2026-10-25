/** Same 1px semantic line and 4px breathing room as the approved Menu Separator. */
export function dividerView(decorative = false): string {
  return `<hr class="ui-divider" data-ui-primitive="divider" ${decorative ? 'aria-hidden="true"' : 'aria-orientation="horizontal"'}>`;
}

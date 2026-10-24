/** Transient dialog visibility only; warning/read/notice/task data belong to WarningStore. */
export class WarningCenterState {
  private expanded = false;
  get open(): boolean { return this.expanded; }
  show(): void { this.expanded = true; }
  hide(): void { this.expanded = false; }
}

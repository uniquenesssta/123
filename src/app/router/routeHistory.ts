/** Right-workspace detail stack. Index -1 is the base page, not browser history. */
export class RouteHistory<Entry> {
  private readonly entries: Entry[] = [];
  private index = -1;

  get current(): Entry | null { return this.entries[this.index] ?? null; }
  get canBack(): boolean { return this.index >= 0; }
  get canForward(): boolean { return this.index + 1 < this.entries.length; }

  push(entry: Entry): void {
    this.entries.splice(this.index + 1);
    this.entries.push(entry);
    this.index = this.entries.length - 1;
  }

  back(): void { if (this.canBack) this.index--; }
  forward(): void { if (this.canForward) this.index++; }
  close(): void { this.index = -1; }
  reset(): void { this.entries.length = 0; this.index = -1; }
}

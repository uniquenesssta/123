/** Transient presentation failure only. No business data, route or persisted preference. */
export class ContentRegionState {
  private failed = false;
  get navigationFailed(): boolean { return this.failed; }
  fail(): void { this.failed = true; }
  clear(): void { this.failed = false; }
}

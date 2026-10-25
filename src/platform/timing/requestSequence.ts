/** A caller-owned latest-result channel. Does not cancel backend work or share feature state. */
export class RequestSequence {
  private revision = 0;
  next(): number { return ++this.revision; }
  invalidate(): void { this.next(); }
  isCurrent(id: number): boolean { return id === this.revision; }
}

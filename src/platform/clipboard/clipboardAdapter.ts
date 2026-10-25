type ClipboardWriter = Pick<Clipboard, "writeText">;

/** Keep the browser permission rejection and receiver; callers own UI feedback. */
export class ClipboardAdapter {
  constructor(private readonly clipboard: () => ClipboardWriter | undefined = () => navigator.clipboard) {}
  async writeText(text: string): Promise<void> {
    const clipboard = this.clipboard();
    if (!clipboard) throw new TypeError("Clipboard is unavailable");
    await clipboard.writeText(text);
  }
}
export const clipboard = new ClipboardAdapter();

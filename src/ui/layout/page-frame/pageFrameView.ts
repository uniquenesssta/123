/** Topbar remains fixed; only the content slot consumes remaining height. */
export function mainFrameView(topbar: string, content: string): string { return `<main class="main-content">${topbar}${content}</main>`; }

import { EventScope } from "../../../app/lifecycle";
/** Manual activation: arrows move focus; native Enter/Space commits through the host. */
export class NavigationKeyboardController {
    private events: EventScope | null = null;
    private closed = false;
    constructor(private readonly root: HTMLElement) { }
    start(): void {
        if (this.closed || this.events)
            return;
        this.events = new EventScope();
        this.events.listen(this.root, "keydown", event => {
            const key = event as KeyboardEvent;
            if (key.altKey || key.ctrlKey || key.metaKey || !(event.target instanceof Element))
                return;
            const target = event.target.closest<HTMLButtonElement>("button"), group = target?.closest<HTMLElement>("[data-ui-navigation]");
            if (!target || !group || !this.root.contains(group) || target.disabled || group.closest("[inert],[hidden]"))
                return;
            if (target.matches("[data-tab-close]"))
                return;
            const items = Array.from(group.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")).filter(b => !b.matches("[data-tab-close]") && b.closest("[data-ui-navigation]") === group && !b.closest("[hidden],[inert]") && b.getClientRects().length > 0);
            const index = items.indexOf(target);
            if (index < 0)
                return;
            if (key.key === "Delete" && target.getAttribute("role") === "tab") {
                const close = target.parentElement?.querySelector<HTMLButtonElement>("[data-tab-close]:not(:disabled)");
                if (close) {
                    key.preventDefault();
                    close.click();
                }
                return;
            }
            const vertical = group.dataset.uiNavigation === "vertical";
            const forward = vertical ? "ArrowDown" : "ArrowRight", backward = vertical ? "ArrowUp" : "ArrowLeft";
            if (![forward, backward, "Home", "End"].includes(key.key))
                return;
            const next = key.key === "Home" ? 0 : key.key === "End" ? items.length - 1 : (index + (key.key === forward ? 1 : -1) + items.length) % items.length;
            key.preventDefault();
            if (group.getAttribute("role") === "tablist")
                for (const item of items)
                    item.tabIndex = item === items[next] ? 0 : -1;
            items[next].focus({ preventScroll: true });
            items[next].scrollIntoView({ block: "nearest", inline: "nearest" });
        });
    }
    destroy(): void { if (this.closed)
        return; this.closed = true; this.events?.destroy(); this.events = null; }
}

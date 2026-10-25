import { spinnerView } from "../spinner";

/** The host owns the async operation. This handle owns only its busy projection. */
export function mountButton(button: HTMLButtonElement) {
  const snapshot = () => ({ html: button.innerHTML, disabled: button.disabled, busy: button.getAttribute("aria-busy"), minWidth: button.style.minWidth });
  let initial = snapshot();
  let destroyed = false, busy = false;
  const restore = () => {
    button.innerHTML = initial.html; button.disabled = initial.disabled;
    if (initial.busy === null) button.removeAttribute("aria-busy"); else button.setAttribute("aria-busy", initial.busy);
    button.style.minWidth = initial.minWidth;
  };
  return {
    setLoading(loading: boolean, label?: string) {
      if (destroyed || loading === busy) return;
      if (loading) {
        if (button.classList.contains("ui-button-ghost")) throw new Error("Ghost has no approved Loading variant");
        initial = snapshot();
        const initialLabel = button.querySelector(".ui-button-label")?.textContent ?? button.textContent ?? "";
        button.style.minWidth = `${button.getBoundingClientRect().width}px`;
        button.disabled = true; button.setAttribute("aria-busy", "true");
        const size = button.dataset.fipControlScale === "48" ? 20 : 16;
        button.innerHTML = spinnerView(size, button.matches(".ui-button-primary,.ui-button-danger") ? "On Accent" : "Default") + '<span class="ui-button-label"></span>';
        button.querySelector(".ui-button-label")!.textContent = label ?? initialLabel;
      } else restore();
      busy = loading;
    },
    destroy() { if (destroyed) return; if (busy) restore(); destroyed = true; },
  };
}

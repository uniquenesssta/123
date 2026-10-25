import { tooltipView } from "./tooltipView";

export function mountTooltip(target: HTMLElement, id: string, text: string) {
  if (target.ownerDocument.getElementById(id)) throw new Error(`Tooltip id already exists: ${id}`);
  const title = target.getAttribute("title"), describedBy = target.getAttribute("aria-describedby");
  const template = target.ownerDocument.createElement("template"); template.innerHTML = tooltipView(id, text);
  const description = template.content.firstElementChild!;
  target.after(description); target.title = text;
  target.setAttribute("aria-describedby", [describedBy, id].filter(Boolean).join(" "));
  let destroyed = false;
  return { destroy() { if (destroyed) return; destroyed = true; description.remove(); if (title === null) target.removeAttribute("title"); else target.title = title; if (describedBy === null) target.removeAttribute("aria-describedby"); else target.setAttribute("aria-describedby", describedBy); } };
}

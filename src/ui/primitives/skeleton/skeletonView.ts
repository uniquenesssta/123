export type SkeletonDensity = "Compact" | "Default";
export function skeletonBlockView(shape: "Text" | "Avatar" | "Rectangle" = "Text", density: SkeletonDensity = "Compact", width: "full" | "half" | "two-thirds" | "column-medium" | "column-small" = "full"): string {
  return `<span class="ui-skeleton-block" data-shape="${shape}" data-density="${density}" data-width="${width}" aria-hidden="true"></span>`;
}

/** Empty/retained/error content is host-owned and never replaced by this first-load projection. */
export function skeletonView(layout: "List" | "Card" | "Table" = "List", density: SkeletonDensity = "Compact"): string {
  const block = (shape: "Text" | "Avatar" | "Rectangle" = "Text", width: "full" | "half" | "two-thirds" | "column-medium" | "column-small" = "full") => skeletonBlockView(shape, density, width);
  const content = layout === "List" ? Array.from({ length: 3 }, () => `<span class="ui-skeleton-row">${block("Avatar")}<span class="ui-skeleton-text-stack">${block()}${block("Text", "half")}</span></span>`).join("") : layout === "Card" ? block("Rectangle") + block() + block("Text", "two-thirds") : Array.from({ length: 4 }, () => `<span class="ui-skeleton-row">${block()}${block("Text", "column-medium")}${block("Text", "column-small")}</span>`).join("");
  return `<span class="ui-skeleton" data-ui-primitive="skeleton" data-layout="${layout}" data-density="${density}" aria-hidden="true">${content}</span>`;
}

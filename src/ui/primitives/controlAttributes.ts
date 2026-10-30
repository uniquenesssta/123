import { requireAccessibleName } from "../accessibility";
import { escapeHtml } from "../../components/format";

export type PrimitiveAttributes = Partial<Record<`data-${string}` | `aria-${string}` | "id" | "name" | "form" | "autofocus" | "role" | "hidden", string | boolean | number>>;

/** Only inert, escaped attributes cross the primitive markup boundary. */
export function primitiveAttributes(attributes: PrimitiveAttributes = {}): string {
  return Object.entries(attributes).map(([name, value]) => {
    if (!/^(?:data-[a-z0-9-]+|aria-[a-z0-9-]+|id|name|form|autofocus|hidden|role)$/.test(name)) throw new Error(`Unsupported primitive attribute: ${name}`);
    if (name.startsWith("aria-") && typeof value === "boolean") return `${name}="${value}"`;
    if (value === undefined || value === false) return "";
    return value === true ? name : `${name}="${escapeHtml(String(value))}"`;
  }).filter(Boolean).join(" ");
}

export function primitiveName(label: string): string {
  return escapeHtml(requireAccessibleName(label));
}

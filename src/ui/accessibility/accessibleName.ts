/** Validate author-provided accessible names; escaping belongs to the markup boundary. */
export function requireAccessibleName(label: string): string {
  if(typeof label!=="string"||!label.trim())throw new Error("A control requires a non-empty accessible name");
  return label;
}

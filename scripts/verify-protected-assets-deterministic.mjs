Object.defineProperty(String.prototype, "localeCompare", {
  configurable: true,
  value(other) {
    const left = String(this);
    const right = String(other);
    if (left < right) return -1;
    if (left > right) return 1;
    return 0;
  },
});

await import("./verify_protected_assets.mjs");

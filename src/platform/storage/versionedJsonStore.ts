import type { PreferenceStore } from "./preferenceStore";

export interface JsonCodec<T> {
  readonly version: number;
  decode(value: unknown): T | null;
  migrate(value: unknown, fromVersion: number): unknown;
}
export interface JsonRead<T> { readonly status: "missing" | "loaded" | "migrated" | "unsupported" | "invalid" | "failed"; readonly value: T | null }

/** Explicit opt-in format; existing raw/plain JSON preferences never pass through this codec. */
export class VersionedJsonStore {
  constructor(private readonly store: PreferenceStore) {}
  read<T>(key: string, codec: JsonCodec<T>): JsonRead<T> {
    try {
      this.checkVersion(codec.version);
      const raw = this.store.getItem(key);
      if (raw === null) return { status: "missing", value: null };
      const envelope: unknown = JSON.parse(raw);
      if (!envelope || typeof envelope !== "object" || Array.isArray(envelope)) return { status: "invalid", value: null };
      const { version, value } = envelope as Record<string, unknown>;
      if (typeof version !== "number" || !Number.isSafeInteger(version) || version < 1) return { status: "invalid", value: null };
      if (version > codec.version) return { status: "unsupported", value: null };
      const decoded = codec.decode(version === codec.version ? value : codec.migrate(value, version));
      if (decoded === null) return { status: "invalid", value: null };
      if (version !== codec.version) this.write(key, decoded, codec);
      return { status: version === codec.version ? "loaded" : "migrated", value: decoded };
    } catch { return { status: "failed", value: null }; }
  }
  write<T>(key: string, value: T, codec: JsonCodec<T>): void {
    this.checkVersion(codec.version);
    const decoded = codec.decode(value);
    if (decoded === null) throw new TypeError("Invalid versioned preference value");
    this.store.writeJson(key, { version: codec.version, value: decoded });
  }
  private checkVersion(version: number): void {
    if (!Number.isSafeInteger(version) || version < 1) throw new TypeError("Invalid preference version");
  }
}

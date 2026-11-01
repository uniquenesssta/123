import type { BootstrapResponse } from "../../types";
import type { PreferenceStore } from "../../platform/storage";
import type { ImportReceipt, ImportRecovery } from "./spreadsheetBatch";
/** Narrow recovery reference only; excludes rows, file path and credentials. */
export function importRecovery(store: PreferenceStore, key: string, snapshot: () => BootstrapResponse | null): ImportRecovery {
  function scope(): string | null {
    const state = snapshot();
    if (!state?.data.database_configured || !state.data.database_url) return null;
    try { const url = new URL(state.data.database_url); return JSON.stringify([url.protocol, url.hostname, url.port, url.pathname, url.username]); } catch { return null; }
  }
  return {
    read() {
      const identity = scope(), record = store.readObject<{ receipts: Record<string,ImportReceipt> }>(key);
      if (!identity) return null;
      const r = record?.receipts?.[identity];
      return r && typeof r.batch_id === "string" && typeof r.source_sha256 === "string" && typeof r.source_file_name === "string" && ["add_only","add_and_update"].includes(r.import_mode) ? r : null;
    },
    save(r) {
      const identity = scope();
      if (!identity) throw new Error("数据库身份未核实，无法建立安全的批次恢复记录");
      const previous=store.readObject<{receipts:Record<string,ImportReceipt>}>(key)?.receipts??{};
      store.writeJson(key, { receipts:{...previous,[identity]:{batch_id:r.batch_id,source_sha256:r.source_sha256,import_mode:r.import_mode,source_file_name:r.source_file_name}} });
    },
    clear() { const identity=scope(),receipts={...store.readObject<{receipts:Record<string,ImportReceipt>}>(key)?.receipts};if(identity&&receipts[identity]){delete receipts[identity];if(Object.keys(receipts).length)store.writeJson(key,{receipts});else store.removeItem(key);} },
  };
}

import type { BootstrapResponse } from "../../types";
import type { PreferenceStore } from "../../platform/storage";
export interface HideReceipt {
    runId: string;
    label: string;
    lineage: string;
}
export interface RunRecovery {
    all(): HideReceipt[];
    read(id: string): HideReceipt | null;
    save(value: HideReceipt): void;
    clear(id: string): void;
}
/** Public identifiers only; credentials never enter a persisted receipt. */
export function runRecovery(store: PreferenceStore, key: string, snapshot: () => BootstrapResponse | null): RunRecovery {
    const identity = () => { const s = snapshot(); if (!s?.data.database_configured || !s.data.database_url)
        return null; const u = new URL(s.data.database_url); return JSON.stringify([u.protocol, u.hostname, u.port, u.pathname, u.username]); };
    const all = () => store.readObject<{
        databases: Record<string, Record<string, HideReceipt>>;
    }>(key)?.databases ?? {};
    const current = () => { const id = identity(); return id ? all()[id] ?? {} : {}; };
    return {
        all() { return Object.values(current()).filter(r => r && typeof r.runId === "string" && typeof r.lineage === "string" && typeof r.label === "string").map(r => structuredClone(r)); },
        read(id) { const r = current()[id]; return r && r.runId === id ? structuredClone(r) : null; },
        save(value) { const id = identity(); if (!id)
            throw Error("数据库身份未核实，不能安全保存移除请求"); store.writeJson(key, { databases: { ...all(), [id]: { ...current(), [value.runId]: structuredClone(value) } } }); },
        clear(runId) { const id = identity(); if (!id)
            return; const databases = { ...all() }, entries = { ...current() }; delete entries[runId]; if (Object.keys(entries).length)
            databases[id] = entries;
        else
            delete databases[id]; if (Object.keys(databases).length)
            store.writeJson(key, { databases });
        else
            store.removeItem(key); }
    };
}

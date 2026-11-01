import type { PreferenceStore } from '../../platform/storage';
import type { BootstrapResponse } from '../../types';
import type { ReviewRecovery, ReviewReceipt } from './reviewPorts';
/** Persist narrow operation identities, never database credentials or workbook contents. */
export function reviewDatabaseIdentity(s: BootstrapResponse | null): string | null {
    if (!s?.data.database_configured || !s.data.database_url)
        return null;
    const u = new URL(s.data.database_url);
    return JSON.stringify([u.protocol, u.hostname, u.port, u.pathname, u.username]);
}
export function reviewRecovery(store: PreferenceStore, key: string, snapshot: () => BootstrapResponse | null): ReviewRecovery {
    const database = () => reviewDatabaseIdentity(snapshot());
    const all = () => store.readObject<Record<string, Record<string, ReviewReceipt>>>(key) ?? {};
    return {
        read(id) { const db = database(), value = db ? all()[db]?.[id] : null; return value && value.matchId === id ? structuredClone(value) : null; },
        save(value) { const db = database(); if (!db)
            throw Error('数据库身份未核实，不能安全保存复盘请求'); const data = all(); store.writeJson(key, { ...data, [db]: { ...data[db], [value.matchId]: structuredClone(value) } }); },
        clear(id) { const db = database(); if (!db)
            return; const data = all(), records = { ...data[db] }; delete records[id]; if (Object.keys(records).length)
            data[db] = records;
        else
            delete data[db]; if (Object.keys(data).length)
            store.writeJson(key, data);
        else
            store.removeItem(key); },
    };
}

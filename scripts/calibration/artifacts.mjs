import fs from 'node:fs';
import crypto from 'node:crypto';
export function canonical(value) {
    if (value === null || typeof value === 'string' || typeof value === 'boolean')
        return JSON.stringify(value);
    if (typeof value === 'number') {
        if (!Number.isFinite(value))
            throw Error('制品包含非有限数值');
        return JSON.stringify(value);
    }
    if (Array.isArray(value))
        return '[' + value.map(canonical).join(',') + ']';
    if (value && typeof value === 'object')
        return '{' + Object.keys(value).sort().map(k => JSON.stringify(k) + ':' + canonical(value[k])).join(',') + '}';
    throw Error('制品包含不支持的值');
}
export const digest = value => crypto.createHash('sha256').update(canonical(value)).digest('hex');
export const seal = body => ({ body, sha256: digest(body) });
export function rejectSecrets(value) {
    if (!value || typeof value !== 'object')
        return;
    for (const [key, child] of Object.entries(value)) {
        if (/^(?:password|api_key|access_token|secret|connection_url|database_url)$/i.test(key) && child !== null && child !== '')
            throw Error('制品包含凭据字段，拒绝落盘');
        rejectSecrets(child);
    }
}
export function openArtifact(file) {
    const bytes = fs.readFileSync(file);
    if (bytes.length > 32 * 1024 * 1024)
        throw Error('单个采样制品超过32MB');
    const row = JSON.parse(bytes.toString('utf8'));
    if (!row.body || typeof row.sha256 !== 'string' || digest(row.body) !== row.sha256)
        throw Error('采样制品摘要不匹配');
    return row;
}
/** Exclusive creation preserves previous snapshots. Failed/incomplete writes never validate as sealed artifacts. */
export function writeArtifact(file, body) {
    rejectSecrets(body);
    const artifact = seal(body), fd = fs.openSync(file, 'wx', 0o600);
    try {
        fs.writeFileSync(fd, canonical(artifact) + '\n', 'utf8');
        fs.fsyncSync(fd);
    }
    finally {
        fs.closeSync(fd);
    }
    return artifact;
}
export function readJson(file) { return JSON.parse(fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, '')); }
export function instant(value, label) {
    const time = typeof value === 'string' ? Date.parse(value) : NaN;
    if (!Number.isFinite(time) || !/(?:Z|[+-]\d\d:\d\d)$/.test(value))
        throw Error(label + '必须包含明确时区');
    return time;
}

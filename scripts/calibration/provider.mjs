import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
export function oracle(executable, args = []) {
    const file = path.resolve(executable);
    if (!fs.statSync(file).isFile())
        throw Error('本机校准提供器路径无效');
    return request => {
        const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !/(?:DATABASE_URL|PGPASSWORD|API_KEY|ACCESS_TOKEN|SECRET)/i.test(key)));
        const result = spawnSync(file, args, {
            input: JSON.stringify({ protocol: 'football.initial-calibration.oracle.v1', ...request }) + '\n', encoding: 'utf8', windowsHide: true, shell: false, timeout: 120000, maxBuffer: 16 * 1024 * 1024, env
        });
        if (result.error)
            throw Error('本机校准提供器执行失败：' + result.error.code);
        if (result.status !== 0)
            throw Error('本机校准提供器拒绝评估：' + String(result.stdout || result.stderr).replace(/postgres(?:ql)?:\/\/[^\s]+/g, '[数据库连接已隐藏]').slice(0, 1200));
        const row = JSON.parse(result.stdout);
        if (row.protocol !== 'football.initial-calibration.oracle.v1' || row.error)
            throw Error('本机提供器协议或评估结果无效');
        if (request.action === 'describe')
            row.executable_sha256 = crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
        return row;
    };
}

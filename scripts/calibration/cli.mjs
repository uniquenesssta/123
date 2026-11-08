import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { readJson, openArtifact, writeArtifact } from './artifacts.mjs';
import { captureBody, outcomeBody, readCohort } from './dataset.mjs';
import { oracle } from './provider.mjs';
import { fitInitial } from './fit.mjs';
const [command, ...args] = process.argv.slice(2), options = {};
const required = key => {
    if (!options[key])
        throw Error('缺少 --' + key);
    return options[key];
};
try {
    for (let i = 0; i < args.length; i += 2) {
        if (!args[i]?.startsWith('--') || args[i + 1] === undefined || Object.hasOwn(options, args[i].slice(2)))
            throw Error('参数必须为不重复的 --名称 值');
        options[args[i].slice(2)] = args[i + 1];
    }
    if (command === 'capture') {
        const parameters = readJson(required('parameters'));
        parameters.profile = readJson(required('profile'));
        const matchId = required('match'), horizon = required('horizon'), dir = path.resolve(required('directory'));
        if (!fs.statSync(dir).isDirectory())
            throw Error('采样目录不存在');
        const result = spawnSync(path.resolve(required('producer')), ['--match', matchId, '--horizon', horizon], {
            encoding: 'utf8', windowsHide: true, shell: false, timeout: 120000, maxBuffer: 32 * 1024 * 1024
        });
        if (result.status !== 0 || result.error)
            throw Error('赛前采样失败：' + String(result.stderr || result.error).replace(/postgres(?:ql)?:\/\/[^\s]+/g, '[数据库连接已隐藏]').slice(0, 1500));
        const body = captureBody(JSON.parse(result.stdout), parameters, required('scope'), { matchId, horizon }), file = path.join(dir, body.capture_id + '.capture.json');
        writeArtifact(file, body);
        console.log(JSON.stringify({
            status: 'CAPTURED', capture_id: body.capture_id, file, scope: body.scope
        }));
    }
    else if (command === 'outcome') {
        const input = path.resolve(required('capture')), capture = openArtifact(input), body = outcomeBody(capture, readJson(required('result'))), file = path.join(path.dirname(input), body.capture_id + '.outcome.json');
        writeArtifact(file, body);
        console.log(JSON.stringify({ status: 'OUTCOME_LINKED', file }));
    }
    else if (command === 'fit') {
        const output = path.resolve(required('output'));
        if (fs.existsSync(output))
            throw Error('结果文件已存在，禁止覆盖');
        const rows = readCohort(path.resolve(required('directory'))), settings = readJson(required('settings')), result = fitInitial(rows, settings, oracle(required('oracle')));
        writeArtifact(output, result);
        console.log(JSON.stringify({
            status: result.status, training_matches: result.training_matches, holdout_matches: result.holdout_matches, alpha_calibrated: false, file: output
        }));
        if (['VALIDATION_FAILED', 'NOT_CONVERGED'].includes(result.status))
            process.exitCode = 2;
    }
    else {
        console.log('独立初始校准：capture（赛前采样）、outcome（关联真实赛果）、fit（本机提供器拟合）。参数及用法见 docs/initial-calibration.md。');
        if (command && command !== 'help')
            process.exitCode = 2;
    }
}
catch (error) {
    console.error(JSON.stringify({
        status: 'BLOCKED', reason: error instanceof Error ? error.message : String(error), alpha_calibrated: false
    }));
    process.exitCode = 1;
}

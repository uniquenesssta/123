import type {IssueLogDraft, IssueSeverity} from '../../types';
import {redactLogText} from './logSummary';

export function userFacingError(error: unknown): string {
  const message = redactLogText(error instanceof Error ? error.message : String(error));
  if (/column .* does not exist|relation .* does not exist|数据库迁移失败/i.test(message)) return '数据库结构版本与客户端不一致。请重启客户端完成自动升级后重试，现有数据不会被删除。';
  if (message.startsWith('PostgreSQL 连接或查询失败')) return '数据库操作失败，请检查数据服务连接状态后重试。';
  return message;
}

interface IssueReporterPorts {
  wasLogged(error:unknown):boolean;
  record(draft: IssueLogDraft): Promise<unknown>;
  critical(operation: string, message: string): void;
}

/** UI errors share the transport's redaction/receipt boundary; issue recording never recursively reports itself. */
export function createClientIssueRecorder(ports: IssueReporterPorts) {
  return (error: unknown, operation: string, severity: IssueSeverity = 'error', traceId?: string): void => {
    const message=userFacingError(error), safeOperation=redactLogText(operation);
    if (severity==='critical') ports.critical(safeOperation,message);
    if (ports.wasLogged(error)) return;
    if (severity!=='critical' && /^(请选择|请先|请填写|请输入|请至少|当前没有可|没有可用于|尚未选择)/.test(message.trim())) return;
    void ports.record({severity,source:severity==='critical'?'global':'frontend',operation:safeOperation,user_message:message,technical_message:redactLogText(error instanceof Error?error.message:String(error)),occurrence_key:traceId??null}).catch(()=>{/* Keep the original UI error when history is unavailable. */});
  };
}

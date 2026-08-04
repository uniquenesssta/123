import { escapeHtml } from "./format";

export function inlineDatabaseSetup(
  title = "连接数据库后继续",
  description = "在当前页面完成连接，不会跳转到其他页面。连接成功后本页会自动恢复。",
  connectionError: string | null = null,
): string {
  return `
    <section class="panel inline-setup-panel" aria-labelledby="inline-database-title">
      <div class="setup-heading">
        <span class="setup-icon" aria-hidden="true">库</span>
        <div><p class="eyebrow">当前页面所需</p><h2 id="inline-database-title">${escapeHtml(title)}</h2><p>${escapeHtml(description)}</p></div>
      </div>
      ${connectionError ? `<div class="alert error"><strong>上次连接未成功</strong><span>${escapeHtml(connectionError)}</span></div>` : ""}
      <div class="form-grid database-inline-form">
        <label class="field field-wide"><span>数据库连接地址</span><input id="database-url" type="password" autocomplete="off" placeholder="postgres://用户名:密码@服务器:5432/数据库名" /><small class="field-note">通常由数据库安装或管理员提供。Windows 下使用当前用户凭据保护，问题日志会自动隐藏密码。</small></label>
        <label class="field"><span>最大连接数</span><input id="max-connections" type="number" min="1" max="100" value="10" /></label>
        <label class="field"><span>连接超时（秒）</span><input id="connect-timeout" type="number" min="1" max="120" value="10" /></label>
      </div>
      <div class="workflow-actions"><span class="field-note">连接时会自动补齐数据库结构，不会删除现有数据。</span><button class="primary" data-action="connect-database">连接并继续本页操作</button></div>
    </section>`;
}

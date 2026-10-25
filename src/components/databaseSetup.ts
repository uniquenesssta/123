import { escapeHtml } from "./format";
import { formFieldView } from "../ui/forms/form-field";
import { formActionsView } from "../ui/forms/form-actions";
import { textInputView } from "../ui/primitives/text-input";
import { numberInputView } from "../ui/primitives/number-input";
import { validationSummaryView } from "../ui/forms/validation-summary";

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
      ${connectionError ? validationSummaryView({ id: "database-connection-errors", title: "上次连接未成功，请检查连接信息。", issues: [{ controlId: "database-url", message: "请检查连接地址；详细原因可在顶栏告警中心查看。" }] }) : ""}
      <div class="form-grid database-inline-form">
        <div class="field-wide">${formFieldView({ id: "database-url", label: "数据库连接地址", hint: "通常由数据库安装或管理员提供。Windows 下使用当前用户凭据保护，问题日志会自动隐藏密码。", control: binding => textInputView({ ...binding, type: "password", autoComplete: "off", placeholder: "postgres://用户名:密码@服务器:5432/数据库名" }) })}</div>
        ${formFieldView({ id: "max-connections", label: "最大连接数", control: binding => numberInputView({ ...binding, min: 1, max: 100, value: "10", showStepper: false }) })}
        ${formFieldView({ id: "connect-timeout", label: "连接超时（秒）", control: binding => numberInputView({ ...binding, min: 1, max: 120, value: "10", showStepper: false }) })}
      </div>
      <div class="workflow-actions"><span class="field-note">连接时会自动补齐数据库结构，不会删除现有数据。</span>${formActionsView({ confirm: { label: "连接并继续本页操作", variant: "primary", attributes: { "data-action": "connect-database" } } })}</div>
    </section>`;
}

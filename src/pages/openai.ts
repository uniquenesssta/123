import { escapeHtml } from "../components/format";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceTaskAnchorNavigation } from "../components/workspace";
import type {
  OpenAiApiProtocol,
  OpenAiProfileSummary,
  OpenAiProfilesState,
} from "../types";

function statusLabel(profile: OpenAiProfileSummary): string {
  if (!profile.has_api_key) return "未保存密钥";
  if (profile.last_test_status === "success") return "连接正常";
  if (profile.last_test_status === "failed") return "连接失败";
  return "尚未测试";
}

function testTime(value: string | null): string {
  if (!value) return "尚未测试";
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toLocaleString("zh-CN", { hour12: false });
}

function protocolLabel(protocol: OpenAiApiProtocol): string {
  return protocol === "responses" ? "Responses" : "Chat Completions";
}

function defaultProfile(): OpenAiProfileSummary {
  const now = new Date().toISOString();
  return {
    id: "",
    name: "新的兼容 API 配置",
    provider: "openai_compatible",
    api_protocol: "responses",
    api_endpoint: "https://api.openai.com/v1/responses",
    token_limit_field: "max_output_tokens",
    api_workspace_web_search_mode: "disabled",
    api_example_template: null,
    formal_research_candidate: true,
    is_active: false,
    has_api_key: false,
    api_key_mask: null,
    api_base_url: "https://api.openai.com/v1",
    research_model: "gpt-5.5",
    extraction_model: "gpt-5.5",
    fallback_model: null,
    reasoning_effort: "medium",
    timeout_seconds: 180,
    max_retries: 3,
    max_concurrency: 2,
    max_output_tokens: 12000,
    max_tool_calls: 12,
    search_context_size: "high",
    last_test_status: "untested",
    last_test_message: null,
    last_tested_at: null,
    created_at: now,
    updated_at: now,
  };
}

export function openAiPage(
  state: OpenAiProfilesState | null,
  selectedProfileId: string | null,
  creating: boolean,
): string {
  const profiles = state?.profiles ?? [];
  const selected = creating
    ? defaultProfile()
    : (profiles.find((profile) => profile.id === selectedProfileId) ??
      profiles.find((profile) => profile.is_active) ??
      profiles[0] ??
      defaultProfile());
  const keyPlaceholder = selected.has_api_key
    ? "已安全保存；留空表示不修改"
    : "粘贴该兼容服务的 API Key";
  const isNew = creating || selected.id.length === 0;
  const readiness =
    selected.api_protocol === "responses"
      ? "AI 问答 · 正式研究候选"
      : "AI 问答可用";

  const activeReady = profiles.some((profile) => profile.is_active && profile.has_api_key);
  const taskNavigation = workspaceTaskAnchorNavigation([
    { id: "openai-profile-list-section", index: "01", label: "配置档案", description: "选择或新建服务配置", badge: `${profiles.length}` },
    { id: "openai-request-settings", index: "02", label: "请求与密钥", description: "协议、端点和密钥" },
    { id: "openai-model-settings", index: "03", label: "模型与研究", description: "研究模型和正式参数" },
    { id: "openai-test-security", index: "04", label: "测试与安全", description: "连接结果和密钥边界", badge: statusLabel(selected) },
  ]);
  return `<section class="module-workspace-page openai-module-workspace">
    ${taskPageHeader({ eyebrow: "兼容 API", title: "兼容 API 设置", description: "支持多个 OpenAI-compatible 配置档案；请求端点、模型参数和安全测试都在当前页面完成，密钥始终由本机凭据管理器保存。", status: { label: activeReady ? "已选择可用配置" : "等待配置", tone: activeReady ? "success" : "warning" }, actions: '<button class="primary" data-action="new-openai-profile">新建配置</button>' })}
    ${taskContextRibbon([
      { label: "当前配置", value: selected.name, note: protocolLabel(selected.api_protocol), tone: selected.is_active ? "accent" : "neutral" },
      { label: "密钥状态", value: selected.has_api_key ? "已安全保存" : "尚未保存", note: "明文不会回传到界面", tone: selected.has_api_key ? "success" : "warning" },
      { label: "连接测试", value: statusLabel(selected), note: testTime(selected.last_tested_at), tone: selected.last_test_status === "success" ? "success" : selected.last_test_status === "failed" ? "danger" : "neutral" },
      { label: "研究资格", value: readiness, note: selected.research_model, tone: selected.formal_research_candidate ? "accent" : "neutral" },
    ])}
    <div class="core-local-navigation">${taskNavigation}</div>
    <div class="openai-module-stage" data-workspace-scroll-key="openai-stage">
    <section class="openai-workbench">
      <aside id="openai-profile-list-section" class="panel openai-profile-sidebar workspace-anchor-target">
        <div class="openai-profile-sidebar-head">
          <div><span>配置档案</span><strong>${profiles.length} 个兼容 API 配置</strong></div>
          <button class="icon-button" data-action="new-openai-profile" title="新建配置" aria-label="新建兼容API配置">＋</button>
        </div>
        <div class="openai-profile-list">
          ${
            profiles
              .map(
                (profile) => `
            <button class="openai-profile-card ${selected.id === profile.id && !creating ? "selected" : ""}" data-action="select-openai-profile" data-profile-id="${escapeHtml(profile.id)}">
              <span class="openai-provider-logo">API</span>
              <div>
                <strong>${escapeHtml(profile.name)}</strong>
                <small>${escapeHtml(protocolLabel(profile.api_protocol))} · ${escapeHtml(profile.research_model)}</small>
              </div>
              <i class="profile-state ${profile.last_test_status} ${profile.has_api_key ? "has-key" : "no-key"}"></i>
              ${profile.is_active ? '<b class="active-badge">当前</b>' : ""}
            </button>
          `,
              )
              .join("") ||
            '<div class="empty-profile-list"><strong>还没有配置</strong><small>点击右上角“＋”创建</small></div>'
          }
        </div>
        <div class="openai-profile-sidebar-foot">
          <small>配置参数保存在本机；每个档案的 API Key 分别存入当前 Windows 用户的凭据管理器。</small>
        </div>
      </aside>

      <div class="panel openai-editor">
        <div class="openai-editor-head">
          <div>
            <span>${isNew ? "新建配置" : "编辑配置"}</span>
            <h2>${escapeHtml(selected.name)}</h2>
          </div>
          <div class="openai-editor-status">
            <span class="connection-state ${selected.last_test_status}">${statusLabel(selected)}</span>
            <span class="protocol-pill ${selected.formal_research_candidate ? "formal" : "test-only"}">${readiness}</span>
            ${selected.is_active ? '<span class="active-pill">当前使用</span>' : ""}
          </div>
        </div>

        <input id="openai-profile-id" type="hidden" value="${escapeHtml(isNew ? "" : selected.id)}" />

        <div id="openai-request-settings" class="openai-editor-section api-example-section workspace-anchor-target">
          <div class="section-label">
            <span>API Example 实时解析</span>
            <small>可直接粘贴服务商文档中的完整 curl；同时包含 Chat 与 Responses 时默认优先选择 Responses。</small>
          </div>
          <label class="field">
            <span>API Example（curl 或包含 url / headers / body 的 JSON）</span>
            <textarea id="openai-api-example" rows="11" spellcheck="false" placeholder="curl https://api.example.com/v1/responses ...">${escapeHtml(selected.api_example_template ?? "")}</textarea>
          </label>
          <div id="openai-api-example-status" class="api-example-status idle">
            <div><strong>等待输入</strong><span>粘贴或编辑后会自动替换下方协议、端点和模型。</span></div>
          </div>
        </div>

        <div class="openai-editor-section">
          <div class="section-label"><span>请求设置</span><small>这些字段会被 API Example 实时更新，也可以继续手工修正。</small></div>
          <div class="field-row">
            <label class="field"><span>配置名称</span><input id="openai-profile-name" value="${escapeHtml(selected.name)}" maxlength="80" autocomplete="off" /></label>
            <label class="field"><span>协议</span><select id="openai-api-protocol">
              <option value="responses" ${selected.api_protocol === "responses" ? "selected" : ""}>Responses</option>
              <option value="chat_completions" ${selected.api_protocol === "chat_completions" ? "selected" : ""}>Chat Completions</option>
            </select></label>
          </div>
          <input id="openai-api-workspace-web-search-mode" type="hidden" value="disabled" />
          <label class="field"><span>完整请求端点</span><input id="openai-api-endpoint" value="${escapeHtml(selected.api_endpoint)}" autocomplete="off" spellcheck="false" /></label>
          <label class="field"><span>API 基础地址</span><input id="openai-api-base-url" value="${escapeHtml(selected.api_base_url)}" autocomplete="off" spellcheck="false" /></label>
          <label class="field openai-key-field">
            <span>API Key</span>
            <div class="secret-input-wrap">
              <input id="openai-api-key" type="password" value="" placeholder="${escapeHtml(keyPlaceholder)}" autocomplete="new-password" spellcheck="false" />
              <button type="button" data-action="toggle-openai-key-visibility">显示</button>
            </div>
            <small>${selected.has_api_key ? "已保存密钥不会回传到界面。输入新值并保存可替换。" : "粘贴包含 Authorization 密钥的示例时会临时填入此框；保存后立即交给 Rust 写入 Windows 凭据管理器。"}</small>
          </label>
        </div>

        <div id="openai-model-settings" class="openai-editor-section workspace-anchor-target">
          <div class="section-label"><span>模型组合</span><small>API Example中的model会同时替换研究模型与提取模型。</small></div>
          <div class="field-row">
            <label class="field"><span>研究模型</span><input id="openai-research-model" value="${escapeHtml(selected.research_model)}" autocomplete="off" spellcheck="false" /></label>
            <label class="field"><span>提取模型</span><input id="openai-extraction-model" value="${escapeHtml(selected.extraction_model)}" autocomplete="off" spellcheck="false" /></label>
          </div>
          <label class="field"><span>备用模型（可留空）</span><input id="openai-fallback-model" value="${escapeHtml(selected.fallback_model ?? "")}" autocomplete="off" spellcheck="false" /></label>
        </div>

        <input id="openai-token-limit-field" type="hidden" value="${escapeHtml(selected.token_limit_field)}" />
        <input id="openai-max-output-tokens" type="hidden" value="${selected.max_output_tokens}" />
        <details class="openai-advanced">
          <summary><div><span>正式研究参数</span><small>供 P4 严格研究链使用；AI 问答只使用超时与重试设置</small></div><b>展开</b></summary>
          <div class="openai-advanced-grid">
            <label class="field"><span>推理强度</span><select id="openai-reasoning-effort">
              ${["none", "minimal", "low", "medium", "high", "xhigh"].map((value) => `<option value="${value}" ${selected.reasoning_effort === value ? "selected" : ""}>${value}</option>`).join("")}
            </select></label>
            <label class="field"><span>搜索上下文</span><select id="openai-search-context-size">
              ${["low", "medium", "high"].map((value) => `<option value="${value}" ${selected.search_context_size === value ? "selected" : ""}>${value}</option>`).join("")}
            </select></label>
            <label class="field"><span>超时（秒）</span><input id="openai-timeout-seconds" type="number" min="10" max="900" value="${selected.timeout_seconds}" /></label>
            <label class="field"><span>最大重试</span><input id="openai-max-retries" type="number" min="0" max="10" value="${selected.max_retries}" /></label>
            <label class="field"><span>最大并发</span><input id="openai-max-concurrency" type="number" min="1" max="16" value="${selected.max_concurrency}" /></label>
            <label class="field"><span>工具调用上限</span><input id="openai-max-tool-calls" type="number" min="1" max="100" value="${selected.max_tool_calls}" /></label>
          </div>
        </details>

        <div id="openai-test-security" class="openai-test-summary workspace-anchor-target ${selected.last_test_status}">
          <div><span>最近连接测试</span><strong>${statusLabel(selected)}</strong></div>
          <p>${escapeHtml(selected.last_test_message ?? "保存后点击“保存并测试”，客户端会向当前完整端点发送最小请求，不再依赖 /models 接口。")}</p>
          <small>${testTime(selected.last_tested_at)}</small>
        </div>

        <div class="openai-editor-actions">
          <div>
            <button class="primary" data-action="save-openai-profile">保存配置</button>
            <button class="secondary" data-action="test-openai-profile">保存并测试</button>
            ${!isNew && !selected.is_active ? '<button class="secondary" data-action="activate-openai-profile">设为当前</button>' : ""}
          </div>
          <div>
            ${!isNew && selected.has_api_key ? '<button class="secondary danger-quiet" data-action="request-clear-openai-key">移除密钥</button>' : ""}
            ${!isNew ? '<button class="secondary danger-quiet" data-action="request-delete-openai-profile">删除配置</button>' : ""}
          </div>
        </div>
      </div>
    </section>

    <section class="panel openai-security-note">
      <div class="security-icon">✓</div>
      <div><strong>密钥安全边界</strong><p>保存的 API Example 会自动脱敏为 YOUR_API_KEY；前端无法读取已经保存的明文密钥；密钥不会写入数据库、日志、localStorage、模型包或导出文件。</p></div>
      <code>${escapeHtml(state?.config_path ?? "openai-profiles.json")}</code>
    </section>
    </div>
  </section>`;
}

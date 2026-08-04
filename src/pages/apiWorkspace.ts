import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../components/icons";
import { workspacePaneToggle, workspaceSectionNavigation } from "../components/workspace";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import type {
  ApiWorkspaceMessageRecord,
  ApiWorkspacePreset,
  ApiWorkspaceSessionDetail,
  ApiWorkspaceSessionRecord,
  BootstrapResponse,
  MatchRecord,
  OpenAiProfilesState,
} from "../types";

function dateTime(value: string): string {
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime())
    ? value
    : parsed.toLocaleString("zh-CN", { hour12: false });
}

function profileOptions(
  profiles: OpenAiProfilesState | null,
  selected: string,
): string {
  return (profiles?.profiles.filter((item) => item.has_api_key) ?? [])
    .map(
      (item) =>
        `<option value="${escapeHtml(item.id)}" ${item.id === selected ? "selected" : ""}>${escapeHtml(item.name)}${item.is_active ? "（当前）" : ""}</option>`,
    )
    .join("");
}

function presetOptions(
  presets: ApiWorkspacePreset[],
  selected: string,
): string {
  return presets
    .filter((item) => !["file_structuring"].includes(item.key))
    .map(
      (item) =>
        `<option value="${escapeHtml(item.key)}" ${item.key === selected ? "selected" : ""}>${escapeHtml(item.category)} · ${escapeHtml(item.title)}</option>`,
    )
    .join("");
}

function matchOptions(matches: MatchRecord[], selected: string | null): string {
  return matches
    .map((match) => {
      const label = `${match.home_team_name} vs ${match.away_team_name} · ${dateTime(match.kickoff_time)}`;
      return `<option value="${escapeHtml(match.id)}" ${match.id === selected ? "selected" : ""}>${escapeHtml(label)}</option>`;
    })
    .join("");
}

function presetPrompts(preset: ApiWorkspacePreset | null): string {
  if (!preset?.suggested_questions.length) return "";
  return `<div class="api-prompt-grid">${preset.suggested_questions.map((question) => `<button type="button" class="api-prompt-card" data-action="use-api-workspace-prompt" data-prompt="${escapeHtml(question)}"><span>示例问题</span><strong>${escapeHtml(question)}</strong></button>`).join("")}</div>`;
}

function legacyAudit(
  detail: ApiWorkspaceSessionDetail,
  message: ApiWorkspaceMessageRecord,
): string {
  const operations = detail.operations.filter(
    (item) => item.message_id === message.id,
  );
  const files = detail.files.filter((item) => item.message_id === message.id);
  if (
    !operations.length &&
    !files.length &&
    Object.keys(message.structured_payload ?? {}).length === 0
  )
    return "";
  return `<details class="api-legacy-audit"><summary>历史结构化记录（只读）</summary><div>
    ${operations.length ? `<p>数据库提案：${operations.length} 条。阶段 1 后不再新建或执行此类提案。</p>` : ""}
    ${files.length ? `<p>历史生成文件：${files.length} 个。记录继续保留用于审计，不再作为常用导出功能。</p>` : ""}
    <p>历史数据未删除；当前 AI 问答只保存普通文本消息。</p>
  </div></details>`;
}

function messageCard(
  message: ApiWorkspaceMessageRecord,
  detail: ApiWorkspaceSessionDetail,
): string {
  const isUser = message.role === "user";
  return `<article class="api-message ${isUser ? "user" : "assistant"}">
    <header><div><span>${isUser ? "你" : "AI"}</span><small>${escapeHtml(dateTime(message.created_at))}${message.model_id ? ` · ${escapeHtml(message.model_id)}` : ""}</small></div>${isUser ? "" : `<button class="ghost tiny" data-action="copy-api-workspace-message" data-message-id="${escapeHtml(message.id)}">复制回答</button>`}</header>
    <div class="api-message-content">${escapeHtml(message.content).replaceAll("\n", "<br>")}</div>
    ${legacyAudit(detail, message)}
  </article>`;
}

function pendingMessageCard(message: string, startedAt: string): string {
  return `<article class="api-message user pending"><header><div><span>你</span><small>${escapeHtml(dateTime(startedAt))} · 正在等待 API</small></div></header><div class="api-message-content">${escapeHtml(message).replaceAll("\n", "<br>")}</div><div class="api-message-pending"><i></i><span>请求正在当前发送区处理。其他页面和历史会话仍可正常使用。</span></div></article>`;
}

function conversation(
  detail: ApiWorkspaceSessionDetail | null,
  pendingMessage: {
    content: string;
    started_at: string;
    session_id: string | null;
  } | null,
): string {
  const visiblePending =
    pendingMessage && pendingMessage.session_id === (detail?.session.id ?? null)
      ? pendingMessage
      : null;
  if ((!detail || detail.messages.length === 0) && !visiblePending) {
    return `<div class="api-conversation-empty"><strong>从一个明确问题开始</strong><span>AI 只进行普通文本问答。资料维护请使用 Excel 工作包，当前页面上下文只能只读附加。</span></div>`;
  }
  const messages =
    detail?.messages.map((message) => messageCard(message, detail)).join("") ??
    "";
  return `${messages}${visiblePending ? pendingMessageCard(visiblePending.content, visiblePending.started_at) : ""}`;
}

function sessionList(
  sessions: ApiWorkspaceSessionRecord[],
  selectedId: string | null,
  search: string,
): string {
  const normalized = search.trim().toLocaleLowerCase("zh-CN");
  const filtered = normalized
    ? sessions.filter((item) =>
        `${item.title} ${item.match_label ?? ""} ${String(item.metadata.context_entity_label ?? "")}`
          .toLocaleLowerCase("zh-CN")
          .includes(normalized),
      )
    : sessions;
  if (filtered.length === 0)
    return `<div class="empty-state compact"><strong>${sessions.length ? "没有匹配会话" : "暂无历史会话"}</strong><span>${sessions.length ? "换一个搜索词。" : "发送第一条消息后会自动保存。"}</span></div>`;
  return filtered
    .map(
      (item) =>
        `<article class="api-session-row ${item.id === selectedId ? "selected" : ""}"><button class="api-session-card" data-action="select-api-workspace-session" data-session-id="${escapeHtml(item.id)}"><span>${escapeHtml(item.match_label ?? (typeof item.metadata.context_entity_label === "string" ? item.metadata.context_entity_label : "通用会话"))}</span><strong>${escapeHtml(item.title)}</strong><small>${item.message_count} 条消息 · ${escapeHtml(dateTime(item.updated_at))}</small></button><button class="ghost tiny danger-quiet" data-action="archive-api-workspace-session" data-session-id="${escapeHtml(item.id)}" data-session-title="${escapeHtml(item.title)}">删除</button></article>`,
    )
    .join("");
}

export function apiWorkspacePage(
  state: BootstrapResponse,
  presets: ApiWorkspacePreset[],
  sessions: ApiWorkspaceSessionRecord[],
  detail: ApiWorkspaceSessionDetail | null,
  profiles: OpenAiProfilesState | null,
  matches: MatchRecord[],
  selectedPresetKey: string,
  selectedProfileId: string,
  selectedMatchId: string | null,
  draftMessage: string,
  sending: boolean,
  pendingMessage: {
    content: string;
    started_at: string;
    session_id: string | null;
  } | null,
  contextEntityType: "team" | "player" | null,
  contextEntityLabel: string | null,
  sessionSearch: string,
  includeContext: boolean,
  activeRequestId: string | null,
  moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "AI 问答", title: "普通文本会话", description: "会话历史需要数据库账本支持；AI 不会联网、生成文件或写入数据库。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "会话账本", value: "数据库未连接", note: "连接后加载普通文本会话历史", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以使用 AI 问答", "连接后会加载普通文本会话历史。", state.connection_error)}</section>`;
  }
  const selectedPreset = presets.find((item) => item.key === (detail?.session.preset_key ?? selectedPresetKey)) ?? presets[0] ?? null;
  const activeProfileId = detail?.session.profile_id ?? selectedProfileId;
  const activeMatchId = detail?.session.match_id ?? selectedMatchId;
  const profileReady = Boolean(profiles?.profiles.some((item) => item.id === activeProfileId && item.has_api_key));
  const locked = Boolean(detail);
  const hasContext = Boolean(activeMatchId || (contextEntityType && contextEntityLabel));
  const section = ["chat", "history"].includes(activeSection) ? activeSection : "chat";
  const sectionNav = workspaceSectionNavigation([
    { id: "chat", index: "01", label: "对话工作台", description: "上下文、消息与发送", badge: sending ? "处理中" : `${detail?.messages.length ?? 0}` },
    { id: "history", index: "02", label: "会话历史", description: "搜索、选择与归档", badge: `${sessions.length}` },
  ], section);
  const contextLabel = activeMatchId
    ? matches.find((item) => item.id === activeMatchId)?.home_team_name + " vs " + matches.find((item) => item.id === activeMatchId)?.away_team_name
    : contextEntityLabel ?? "未附加";

  return `<section class="module-workspace-page ai-module-workspace" data-legacy-module-sidebar-state="${moduleSidebarCollapsed ? "collapsed" : "expanded"}">
    ${taskPageHeader({
      eyebrow: "AI 问答",
      title: "普通文本会话",
      description: "当前页面只负责提问与回答；配置、只读上下文、历史会话和发送状态均明确分层。",
      status: { label: sending ? "AI 正在处理" : profileReady ? "可以发送" : "等待 API 配置", tone: sending ? "accent" : profileReady ? "success" : "warning" },
      actions: `<button class="primary" data-action="new-api-workspace-session">新建会话</button><button class="secondary" data-action="refresh-api-workspace">${icon("refresh")}<span>刷新</span></button>`,
    })}
    ${taskContextRibbon([
      { label: "当前会话", value: detail?.session.title ?? "新会话", note: detail ? `${detail.messages.length} 条消息` : "首次发送后自动保存", tone: detail ? "accent" : "neutral" },
      { label: "API 配置", value: profiles?.profiles.find((item) => item.id === activeProfileId)?.name ?? "未选择", note: profileReady ? "密钥可用" : "前往兼容 API 保存配置和密钥", tone: profileReady ? "success" : "warning" },
      { label: "只读上下文", value: contextLabel || "未附加", note: includeContext && hasContext ? "下一条消息将附加摘要" : "当前不会附加资料", tone: includeContext && hasContext ? "accent" : "neutral" },
      { label: "运行边界", value: "纯文本问答", note: "不联网、不写库、不生成文件", tone: "neutral" },
    ])}
    <div class="core-local-navigation">${sectionNav}</div>
    <div class="module-workspace-stage ai-module-stage">
      <section class="workspace-module-view ${section === "chat" ? "active" : ""}" data-workspace-section="chat">
        <div class="ai-chat-layout ${inspectorCollapsed ? "inspector-collapsed" : ""}">
          <main class="ai-chat-main" data-workspace-scroll-key="api-main">
            <section class="panel api-context-panel">
              <div class="api-runtime-note"><div><strong>纯文本边界</strong><p>新请求只发送模型、普通消息和可选只读上下文。运行日志只记录协议、端点、状态、延迟、错误和文本长度，不记录问题正文或结构化 Schema。</p></div><code>${escapeHtml(state.runtime_log_path)}</code></div>
              ${contextEntityType && contextEntityLabel ? `<div class="api-entity-context"><span>${contextEntityType === "team" ? "当前球队上下文" : "当前球员上下文"}</span><strong>${escapeHtml(contextEntityLabel)}</strong><small>仅在勾选后作为只读摘要附加；AI 无法修改它。</small></div>` : ""}
              <div class="api-context-grid">
                <label class="field"><span>API 配置</span><select id="api-workspace-profile" ${locked ? "disabled" : ""}><option value="">请选择兼容 API 配置</option>${profileOptions(profiles, activeProfileId)}</select><small>${profileReady ? "配置和密钥可用" : "请先在“兼容 API”中保存 Responses 或 Chat Completions 配置和密钥"}</small></label>
                <label class="field"><span>问答类型</span><select id="api-workspace-preset" ${locked ? "disabled" : ""}>${presetOptions(presets, selectedPreset?.key ?? "")}</select><small>${escapeHtml(selectedPreset?.description ?? "")}</small></label>
                <label class="field"><span>比赛上下文</span><select id="api-workspace-match" ${locked ? "disabled" : ""}><option value="">不绑定比赛</option>${matchOptions(matches, activeMatchId)}</select><small>可选；只读取客户端已保存的数据，不联网补全。</small></label>
              </div>
              <label class="api-context-toggle"><input id="api-workspace-include-context" type="checkbox" ${includeContext && hasContext ? "checked" : ""} ${hasContext ? "" : "disabled"}/><span><strong>附加当前只读上下文</strong><small>${hasContext ? "将当前球队、球员或比赛摘要附加到下一条问题。" : "当前没有可附加的球队、球员或比赛上下文。"}</small></span></label>
              ${detail ? `<div class="api-session-lock"><span>会话身份已锁定</span><strong>${escapeHtml(detail.session.title)}</strong><small>继续对话时保持原 API 配置、问答类型和上下文身份。</small></div>` : presetPrompts(selectedPreset)}
            </section>
            <section class="panel api-chat-panel">${sending ? `<div class="api-local-progress" role="status"><i></i><div><strong>AI 正在处理当前消息</strong><span>只锁定发送区；你可以切换页面或浏览其他会话。</span></div>${activeRequestId ? `<button class="secondary tiny" data-action="cancel-api-workspace-request" data-request-id="${escapeHtml(activeRequestId)}">取消请求</button>` : ""}</div>` : ""}<div class="api-conversation" id="api-workspace-conversation">${conversation(detail, pendingMessage)}</div><div class="api-composer"><textarea id="api-workspace-message" rows="5" placeholder="输入普通文本问题。需要更新球队、球员、比赛或阵容资料时，请使用 Excel 工作包。">${escapeHtml(draftMessage)}</textarea><div class="api-composer-actions"><span>消息会保存到当前会话；AI 不会执行客户端操作。</span><div class="button-row compact"><button class="ghost" data-action="clear-api-workspace-draft">清空输入</button><button class="primary" data-action="send-api-workspace-message" ${profileReady && !sending ? "" : "disabled"}>${sending ? "正在等待 AI…" : "发送问题"}</button></div></div></div></section>
          </main>
          <aside class="panel workspace-inspector" data-workspace-panel="api-inspector">${workspacePaneToggle("inspector", inspectorCollapsed)}<div class="panel-heading"><div><span>会话检查器</span><h2>${escapeHtml(detail?.session.title ?? "新会话")}</h2></div></div><div class="inspector-kpis"><div><span>历史会话</span><strong>${sessions.length}</strong></div><div><span>消息</span><strong>${detail?.messages.length ?? 0}</strong></div><div><span>请求</span><strong>${sending ? "处理中" : "空闲"}</strong></div></div><p>上下文只读；无联网、无文件生成、无数据库写入。</p><button class="ghost" data-action="reset-current-workspace">清空当前工作区状态</button></aside>
        </div>
      </section>
      <section class="workspace-module-view ${section === "history" ? "active" : ""}" data-workspace-section="history">
        <div class="ai-history-layout">
          <aside class="panel api-session-sidebar" data-workspace-panel="api-sidebar"><div class="panel-heading"><div><span>历史会话</span><h2>${sessions.length} 个会话</h2></div><button class="primary tiny" data-action="new-api-workspace-session">新建</button></div><label class="module-search"><span class="visually-hidden">搜索会话</span>${icon("search")}<input id="api-workspace-session-search" value="${escapeHtml(sessionSearch)}" placeholder="搜索标题、球队或比赛" /></label><div class="api-session-list">${sessionList(sessions, detail?.session.id ?? null, sessionSearch)}</div></aside>
          <main class="panel api-history-preview"><div class="workspace-section-heading"><div><span>会话预览</span><h2>${escapeHtml(detail?.session.title ?? "选择一个历史会话")}</h2><p>${detail ? "查看消息后可返回对话工作台继续提问。" : "从左侧选择会话；不会改变其原 API 配置与上下文身份。"}</p></div>${detail ? '<button class="primary" data-action="select-workspace-section" data-section-id="chat">继续对话</button>' : ""}</div>${detail ? `<div class="api-conversation">${conversation(detail, null)}</div>` : '<div class="empty-state"><strong>尚未选择会话</strong><span>选择一项后在这里预览全部消息。</span></div>'}</main>
        </div>
      </section>
    </div>
  </section>`;
}

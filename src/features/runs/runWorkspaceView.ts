import { escapeHtml } from "../../components/format";
import { inlineDatabaseSetup } from "../../components/databaseSetup";
import { buttonView } from "../../ui/primitives/button";
import { statCardView } from "../../ui/data-display/stat-card";
import { predictionViewData, predictionDetailBody } from "../prediction/index";
import type { BootstrapResponse } from "../../types";
import type { RunsFeature } from "./runsFeature";
import { runHistoryMarkup } from "./runHistoryView";
import { finite, matchName, probability, readiness, recorded, time } from "./runPresentation";
const html = escapeHtml;
const button = (label: string, action: string, disabled = false, runId?: string) => buttonView({ label, variant: "secondary", disabled, attributes: { "data-action": action, ...(runId ? { "data-run-id": runId } : {}) } });
function panel(title: string, description: string, body: string, classes = ""): string { return `<section class="panel runs-panel ${classes}"><header><h2>${html(title)}</h2><p>${html(description)}</p></header>${body}</section>`; }
function detail(view: RunsFeature["view"]): string {
    const id = view.selectedId!;
    if (view.detailLoading)
        return panel("正在载入运行详情", "正在读取所选原记录，不显示上一条记录的结果。", '<p role="status">正在读取…</p>');
    if (view.detailError || !view.detail)
        return panel("运行详情未载入", "原记录内容尚未核实；不会使用上一条记录替代。", `<p role="alert">${html(view.detailError ?? "未返回运行详情")}</p>${button("重新载入", "runs-retry-detail", false, id)}`);
    const doc = view.detail, data = predictionViewData(doc, { stored: true }), row = view.rows.find(r => r.id === id), name = row ? matchName(row) : recorded(doc.match_key, "未命名比赛"), state = readiness(data.inputAudit.readiness_level), top = data.scorelines[0], isHidden = typeof doc.history_hidden_at === "string";
    const facts = [["模型 / 规则", `${data.modelId} · ${data.rulePackage}`], ["数据窗口 / 审计分数", `${recorded(data.snapshotType, "时点未记录")} · ${state.label} / ${finite(data.inputAudit.readiness_score) === null ? "分数未记录" : data.inputAudit.readiness_score + "/100"}`], ["开球 / 运行创建", `${time(row?.kickoff_time)?.short ?? "开球未记录"} / ${time(data.createdAt)?.short ?? "创建时间未记录"}`], ["输入指纹", recorded(data.inputAudit.input_manifest_sha256, "指纹未记录")]];
    return `${panel(name, "只读结果；原运行的输入就绪状态不代表当前提供器可用，也不授予重新执行权限。", `<dl class="runs-facts">${facts.map(([key, value]) => `<div><dt>${html(key)}</dt><dd>${html(value)}</dd></div>`).join("")}</dl>`, "runs-identity")}
  <section class="runs-metrics" aria-label="原运行结果">${statCardView({ label: "最可能比分", value: top?.score ?? null, note: top ? `该单个比分概率 ${probability(top.probability)}` : "暂无矩阵" })}${statCardView({ label: "主胜", value: probability(data.homeWin), note: "原运行保存值" })}${statCardView({ label: "平局", value: probability(data.draw), note: "历史结果，不授予执行许可" })}${statCardView({ label: "客胜", value: probability(data.awayWin), note: "缺失值保持未记录" })}</section>
  ${panel("技术追踪与记录保留", "技术ID、来源和原始内容按需展开；缺失字段不会补成默认模型或0。", `<p>从列表移除只隐藏该运行的列表项。模型运行、概率矩阵、输入快照、复盘与收敛血缘保持保留。</p><div class="button-row">${button("展开技术追踪", "runs-technical", false, id)}${button(isHidden ? "已从列表移除" : "从列表移除本记录", "request-hide-run-history", isHidden || view.locked.includes(id) || view.pending.some(r => r.runId === id), id)}</div>${isHidden ? '<p class="runs-hidden-state" role="status">该记录已隐藏，仍可按原ID追溯完整内容。</p>' : ""}`)}`;
}
export function runsPage(state: BootstrapResponse, view?: RunsFeature["view"]): string {
    const current = view ?? { rows: state.data.recent_runs, selectedId: null, detail: null, detailLoading: false, detailError: null, listLoading: false, firstLoad: false, listError: null, notice: null, pending: [], locked: [] };
    const detailOpen = !!current.selectedId;
    const status = current.listLoading ? current.rows.length || !current.firstLoad ? "正在刷新，保留已载入记录" : "首次读取记录中" : `最近 ${current.rows.length} 条已载入记录`;
    return `<section class="runs-workspace core-workspace-page"><header class="runs-heading"><h1 id="runs-title" tabindex="-1">${detailOpen ? "运行详情" : "历史推演结果"}</h1><p>模型、窗口与结果按记录保存；列表移除不删除底层运行、矩阵、快照或复盘血缘。</p></header>
  ${!state.data.database_configured ? inlineDatabaseSetup("连接数据服务以读取推演记录", "连接成功后可在本页读取原记录。", state.connection_error) : `<div class="runs-toolbar">${detailOpen ? `<div class="button-row">${button("返回记录", "runs-back")}<span>当前运行：${html(current.selectedId)}</span></div>` : `<span role="status">${current.firstLoad && current.listLoading && !current.rows.length ? "已载入记录：—" : status}</span>`}${button("刷新记录", "refresh-runs", current.listLoading)}</div><main class="runs-viewport" data-workspace-scroll-key="runs-main" aria-busy="${current.listLoading || current.detailLoading}">${current.listError ? `<div class="alert error" role="alert"><b>操作未完成，保留原对象</b><span>${html(current.listError)}</span></div>` : ""}${current.notice ? `<p class="runs-notice" role="status">${html(current.notice)}</p>` : ""}${current.pending.length ? `<div class="alert error" role="status"><b>${current.pending.length} 条移除请求结果待核实</b><span>点击“刷新记录”会按原ID查回实际可见性，不会自动重新提交移除。</span></div>` : ""}${detailOpen ? detail(current) : panel("最近运行记录", "时间优先显示开球时间，缺失时标注为创建时间；审计标签不等于重新运行许可。", current.firstLoad && current.listLoading && !current.rows.length ? '<p role="status">正在首次读取记录，条数尚未确定。</p>' : runHistoryMarkup(current.rows, false, [...current.locked, ...current.pending.map(r => r.runId)]), "run-history-panel balanced-history-panel")}<p class="runs-footnote">— 表示未记录；暂无矩阵不等于0%；模型和规则缺失时不猜测补值。</p></main>`}</section>`;
}
export function runTechnicalBody(doc: Record<string, unknown>): string { return predictionDetailBody(doc, { stored: true }); }

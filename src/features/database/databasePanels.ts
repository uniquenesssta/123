import type { BootstrapResponse } from "../../types";
import { escapeHtml } from "../../components/format";
import { redactLogText } from "../../platform/diagnostics";
import { dataTableView } from "../../ui/data-display/data-table";
import { databaseIdentity } from "./databaseOverview";

export function databaseConnection(state: BootstrapResponse): string {
  return `${databaseIdentity(state)}<div class="database-connection-layout"><article class="panel"><h2>连接参数</h2><p>提交时验证地址、连接池和超时；密码不会写入工作区。</p><label class="field"><span>连接地址</span><input id="database-url" type="password" autocomplete="off" placeholder="postgres://football_app:password@localhost:5432/football_model" /></label><div class="field-row"><label class="field"><span>最大连接数</span><input id="max-connections" type="number" min="1" max="100" value="10" /></label><label class="field"><span>连接超时（秒）</span><input id="connect-timeout" type="number" min="1" max="120" value="10" /></label></div><p class="field-note">连接后自动执行未完成的数据库迁移。正式业务数据保存在数据库服务器，本机只保存受保护的连接凭据。</p></article><article class="panel"><h2>连接与保存不是只读操作</h2><p>先建立连接，再执行迁移和内置资料注册，最后保存受保护凭据。</p><p>失败后根据明确结果恢复；不会把未知结果当作成功，也不会自动清空数据。</p><button class="secondary" data-action="show-database-technical">技术连接信息</button></article></div><footer class="database-panel-actions"><button class="primary" data-action="connect-database">连接并保存</button><button class="secondary danger-quiet" data-action="disconnect-database" ${state.data.database_configured ? "" : "disabled"}>清除连接</button></footer>`;
}
export function databaseStatistics(state: BootstrapResponse): string {
  const stats = state.data.stats, prefix = stats?.large_counts_are_estimates ? "约 " : "";
  return `${databaseIdentity(state)}<article class="panel"><h2>核心表数据统计</h2><p>估计数量保留约数标记；未取得的统计不补成 0。</p>${dataTableView({label:"核心表数据统计",columns:[{key:"label",label:"数据类别"},{key:"value",label:"数量",type:"Number"}],rows:[{id:"stat-0",cells:{label:"赛事",value:`${stats?.competitions ?? "—"}`}},{id:"stat-1",cells:{label:"球队",value:`${prefix}${stats?.teams ?? "—"}`}},{id:"stat-2",cells:{label:"球员",value:`${prefix}${stats?.players ?? "—"}`}},{id:"stat-3",cells:{label:"比赛",value:`${prefix}${stats?.matches ?? "—"}`}},{id:"stat-4",cells:{label:"外部数据来源",value:`${stats?.data_providers ?? "—"}`}},{id:"stat-5",cells:{label:"状态记录",value:`${prefix}${stats?.availability_records ?? "—"}`}},{id:"stat-6",cells:{label:"球员能力记录",value:`${prefix}${stats?.ability_observations ?? "—"}`}},{id:"stat-7",cells:{label:"有效阵容",value:`${stats?.active_lineups ?? "—"}`}},{id:"stat-8",cells:{label:"规则包",value:`${stats?.rule_packages ?? "—"}`}},{id:"stat-9",cells:{label:"有效模型规则",value:`${stats?.route_bindings ?? "—"}`}},{id:"stat-10",cells:{label:"推演记录",value:`${prefix}${stats?.model_runs ?? "—"}`}},{id:"stat-11",cells:{label:"待审核更新",value:`${stats?.pending_ability_updates ?? "—"}`}}]})}</article>`;
}
export function databaseDanger(state: BootstrapResponse): string {
  return `${databaseIdentity(state)}<article class="panel database-danger-zone" aria-labelledby="database-reset-title"><h2 id="database-reset-title">彻底清空当前数据库</h2><p>删除当前数据库中的全部球队、球员、比赛、阵容、推演、P4 快照、规则、导入批次、AI 会话、审核记录、任务、审计和设置数据，并重新建立一套空白数据库结构。</p><p>数据库本身和本机连接配置会保留。此操作不可撤销，无法从客户端恢复。</p><p>提交前必须再次核对当前数据库身份，并输入数据库名称确认。</p></article><footer class="database-panel-actions"><button class="primary danger-action" data-action="request-reset-database" ${state.data.database_health ? "" : "disabled"}>彻底清空数据库</button></footer>`;
}
export function databaseTechnical(state: BootstrapResponse): string {
  return `<dl class="detail-list"><div><dt>本机配置位置</dt><dd>${escapeHtml(state.config_path)}</dd></div><div><dt>连接地址（已脱敏）</dt><dd>${escapeHtml(redactLogText(state.data.database_url ?? "—"))}</dd></div></dl>`;
}

import type { LineupBuilderFormState, LineupBuilderPlayer, PlayerCatalogReferenceData, PlayerListItem } from '../../types';
import type { ReviewFeature } from './reviewFeature';
import { ReviewFacts, reviewMetrics } from './reviewFacts';
import { lineupCompletionWorkspace } from '../../components/lineupBuilder';
import { html, reviewPanel, reviewText, reviewNumeric, reviewButton } from './reviewViewParts';
import { iconSlot } from '../../ui/icons/iconSlot';
export interface ReviewLineupSupport {
    references: PlayerCatalogReferenceData | null;
    players: PlayerListItem[];
    builderPlayers: LineupBuilderPlayer[];
    builderForm: LineupBuilderFormState;
}
export function reviewManualView(feature: ReviewFeature, facts: ReviewFacts, support: ReviewLineupSupport): string {
    const match = feature.selected;
    if (!match)
        return reviewPanel('尚未载入比赛', '请返回九步复盘选择需要处理的比赛。', '');
    const state = feature.state, values = facts.values(match, state.lineups, state.detail);
    const input = reviewPanel('备用：手动录入赛果与球员评分', '此入口生成复盘，不自动推进资料包确认、结算或能力回写。', `<div class="review-result-fields">${reviewNumeric('主队90分钟进球', 'home', values.fields.home)}${reviewNumeric('客队90分钟进球', 'away', values.fields.away)}${reviewNumeric('数据覆盖率0–1', 'coverage', values.fields.coverage, 1, .01)}</div><p>未核实的比分、分钟和量化指标保持空白；真实0可明确录入。评分范围0–10。</p><label class="review-field"><span>赛果确认时间</span><span class="review-date-control"><input id="review-finalized" data-review-fact="finalized" type="datetime-local" value="${html(values.fields.finalized)}" aria-label="赛果确认时间">${iconSlot('calendar', 16)}</span></label>${reviewText('复盘版本（留空自动生成追踪版本）', 'version', values.fields.version)}${reviewText('备注与数据来源', 'notes', values.fields.notes)}`, 'review-input-panel');
    const teams = state.lineups.map(lineup => reviewPanel(`${lineup.team_name} · ${lineup.lineup_type === 'actual' ? '实际阵容' : lineup.lineup_type === 'confirmed' ? '确认阵容' : '预计阵容'}`, `${lineup.starter_count} 名首发；手工提交需要两队实际阵容。`, `<div class="review-player-list">${lineup.players.map(player => {
        const raw = values.players[player.player_id];
        return `<article class="review-player-row" data-review-player data-player-id="${html(player.player_id)}" data-team-id="${html(lineup.team_id)}"><div class="review-player-main"><div class="review-player-name"><strong>${html(player.player_name)}</strong><small>${player.is_starter ? '首发' : '替补'} · ${html(player.position_code ?? '位置未记录')} · ${html(player.role_code ?? '角色待补')} · ${player.role_origin === "player_position_default" ? `资料继承（${html(player.role_source_position_code ?? "位置未记录")}）` : player.role_origin === "lineup_override" ? "本场覆盖" : "角色缺失"}</small></div>${reviewNumeric('分钟', 'minutes', raw.minutes, 150, 1, player.player_id)}${reviewNumeric('评分0–10', 'rating', raw.rating, 10, .1, player.player_id)}${reviewNumeric('进球', 'goals', raw.goals, undefined, 1, player.player_id)}${reviewNumeric('助攻', 'assists', raw.assists, undefined, 1, player.player_id)}${reviewButton("补充数据", "toggle-review-metrics", false, false, { "data-player-id": player.player_id, "aria-expanded": String(raw._expanded === "true"), "aria-controls": "review-metrics-" + player.player_id })}</div><details class="review-player-extra" id="review-metrics-${html(player.player_id)}" ${raw._expanded === "true" ? "open" : ""}><summary>量化指标与数据来源</summary><p>所有数值来自实际录入或已保存事实；空白不会补0。换人和结构化事件请使用资料包录入；已有事件保留原状态。</p><div class="review-extra-grid">${reviewMetrics.slice(2).map(([key, label, step]) => reviewNumeric(label, key, raw[key], undefined, step, player.player_id)).join('')}</div></details></article>`;
    }).join('')}</div>`, 'review-team-input')).join('');
    const missing = state.lineups.filter(row => row.lineup_type === 'actual').length < 2 ? reviewPanel('补录实际阵容', '保留原有阵容补录入口；保存后会重新读取本场实际阵容。', lineupCompletionWorkspace(support.references, support.players, support.builderPlayers, support.builderForm, match.match_record.id, [match.match_record.home_team_id, match.match_record.away_team_id]), 'review-lineup-completion') : '';
    return `<fieldset class="review-manual-fields" data-workspace-persist="false" ${feature.busy ? 'disabled' : ''}>${input}${teams}${missing}</fieldset>`;
}
export function reviewManualFooter(feature: ReviewFeature, facts: ReviewFacts): string {
    const result = facts.validate(feature.selected, feature.state.lineups, feature.state.detail), allowed = feature.allows('manual');
    return `<footer class="review-manual-submit"><p id="review-manual-validation" role="status">${html(feature.pending ? '原请求结果待核实，请刷新查回后再操作。' : result.error ?? (!allowed ? '请沿已确认的资料包完成事实与复盘生成。' : '所需事实已录入；确认后生成本次人工复盘。'))}</p><div class="button-row">${reviewButton('生成复盘', 'generate-match-review', !!result.error || !allowed, true)}${reviewButton('返回九步流程', 'review-view', false, false, { 'data-review-view': 'workflow' })}</div></footer>`;
}

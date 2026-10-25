import { escapeHtml, formatPercent } from "../../components/format";
import { timelineView } from "../../ui/data-display/timeline";
import type { MatchReviewDetail } from "../../types";

function eventTypeLabel(value: string): string {
  return ({
    substitution: "换人",
    goal: "进球",
    own_goal: "乌龙球",
    assist: "助攻",
    penalty_goal: "点球命中",
    penalty_missed: "点球罚失",
    yellow_card: "黄牌",
    second_yellow_card: "第二张黄牌",
    red_card: "红牌",
    injury: "伤退/伤情",
    var: "VAR",
    formation_change: "阵型变化",
    goalkeeper_change: "门将更换",
    other: "其他事件",
  } as Record<string, string>)[value] ?? value;
}

function eventMinute(minute: number, stoppage: number | null): string {
  return stoppage && stoppage > 0 ? `${minute}+${stoppage}′` : `${minute}′`;
}

export function matchEventTimelineView(detail: MatchReviewDetail): string {
  const summary = detail.event_summary;
  const latestScore = summary.latest_home_score === null || summary.latest_away_score === null
    ? "未逐事件记录"
    : `${summary.latest_home_score}-${summary.latest_away_score}`;
  const timeline = timelineView({ label: "事件时间线", emptyTitle: "尚未录入结构化比赛事件", emptyDescription: "资料包可记录进球、助攻、牌、换人、伤退、VAR 与阵型变化。", items: detail.events.map(event => ({
    id: event.event_key || event.id,
    time: eventMinute(event.minute, event.stoppage_minute), title: eventTypeLabel(event.event_type),
    description: [[event.player_name, event.related_player_name].filter(Boolean).join(" → ") || event.team_name || "未关联主体", event.description].filter(Boolean).join(" · "),
    metadata: `${event.team_name ?? "全场事件"} · 可信度 ${formatPercent(event.confidence)} · 序号 ${event.sequence_no}`,
    score: event.home_score === null || event.away_score === null ? undefined : `${event.home_score}-${event.away_score}`,
    verification: ({verified:"Verified",unverified:"Unverified",disputed:"Disputed"} as const)[event.verification_status],
    revision: ({active:"Original",corrected:"Corrected",cancelled:"Cancelled",superseded:"Superseded"} as const)[event.revision_status],
  })) });
  return `<section class="match-event-panel"><header><div><span>结构化比赛事实</span><h3>事件时间线</h3></div><small>${summary.effective_count} 条有效 · ${summary.verified_count} 条已核验</small></header><div class="match-event-summary"><div><span>当前事件</span><strong>${summary.total_count}</strong></div><div><span>取消/争议</span><strong>${summary.cancelled_count}/${summary.disputed_count}</strong></div><div><span>最后事件分钟</span><strong>${summary.last_event_minute ?? "—"}</strong></div><div><span>最后事件后比分</span><strong>${escapeHtml(latestScore)}</strong></div></div>${timeline}</section>`;
}

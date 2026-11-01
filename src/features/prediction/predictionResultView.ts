import {modelDisplayName} from "./predictionLabels";
import {escapeHtml,formatPercent} from "../../components/format";
import {predictionViewData,type PredictionViewData} from "./predictionProjection";
function stringValueFrom(
  record: Record<string, unknown>,
  key: string,
  fallback = "",
): string {
  const candidate = record[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : fallback;
}

function numberValueFrom(
  record: Record<string, unknown>,
  key: string,
): number | null {
  const candidate = record[key];
  return typeof candidate === "number" && Number.isFinite(candidate)
    ? candidate
    : null;
}

function probabilityMeter(
  label: string,
  value: number | null,
  emphasized = false,
): string {
  if(value===null)return '<div class="probability-meter"><div><span>'+escapeHtml(label)+'</span><strong>未记录</strong></div><i></i></div>';
  const percent = Math.max(0, Math.min(100, value * 100));
  return `<div class="probability-meter ${emphasized ? "emphasized" : ""}"><div><span>${escapeHtml(label)}</span><strong>${formatPercent(value)}</strong></div><i><b style="width:${percent.toFixed(2)}%"></b></i></div>`;
}

function predictionTechnicalDetails(data: PredictionViewData): string {
  const readinessLevel = stringValueFrom(data.inputAudit, "readiness_level", "not_assessed");
  const readinessScore = numberValueFrom(data.inputAudit, "readiness_score");
  const manifestSha = stringValueFrom(data.inputAudit, "input_manifest_sha256", "未记录");
  const inputSha = stringValueFrom(data.inputAudit, "input_sha256", "未记录");
  const readinessLabel = readinessLevel === "formal_ready"
    ? "正式就绪"
    : readinessLevel === "ready_with_warnings"
      ? "带警告"
      : readinessLevel === "shadow_only"
        ? "仅影子"
        : readinessLevel === "blocked"
          ? "已阻断"
          : "未评估";
  return `<details class="technical-details prediction-technical"><summary>技术追踪与输入审计</summary><dl class="detail-facts compact">
    <div><dt>运行编号</dt><dd>${escapeHtml(data.runId)}</dd></div>
    <div><dt>模型版本</dt><dd>${escapeHtml(data.modelVersion)}</dd></div>
    <div><dt>参数版本</dt><dd>${escapeHtml(data.parameterVersion)}</dd></div>
    <div><dt>规则包版本</dt><dd>${escapeHtml(data.rulePackageVersion)}</dd></div>
    <div><dt>输入完整度</dt><dd>${escapeHtml(readinessLabel)}${readinessScore === null ? "" : ` · ${readinessScore}/100`}</dd></div>
    <div><dt>输入清单指纹</dt><dd title="${escapeHtml(manifestSha)}">${escapeHtml(manifestSha)}</dd></div>
    <div><dt>完整输入 SHA256</dt><dd title="${escapeHtml(inputSha)}">${escapeHtml(inputSha)}</dd></div>
    <div><dt>审计版本</dt><dd>${escapeHtml(stringValueFrom(data.inputAudit, "audit_version", "未记录"))}</dd></div>
  </dl></details>`;
}

export function predictionDetailBody(payload: unknown): string {
  const data = predictionViewData(payload);
  const topScore = data.scorelines[0] ?? null;
  const scoreParts = topScore?.score.split("-") ?? ["—", "—"];
  const outcomeMax = Math.max(...[data.homeWin,data.draw,data.awayWin].filter((x):x is number=>x!==null));
  const auditStatus = stringValueFrom(
    data.audit,
    "probability_sum_status",
    "未记录",
  );
  const coverageStatus = stringValueFrom(data.audit, "coverage_status", "未记录");
  return `<div class="prediction-detail">
    <section class="prediction-detail-hero">
      <div class="prediction-detail-team"><span>主队</span><strong>${escapeHtml(data.homeTeam)}</strong><small>${data.homeLambda === null ? "预计进球暂无" : `预计进球 ${data.homeLambda.toFixed(2)}`}</small></div>
      <div class="prediction-detail-score"><span>最可能比分</span><div><b>${escapeHtml(scoreParts[0] ?? "—")}</b><i>:</i><b>${escapeHtml(scoreParts[1] ?? "—")}</b></div><small>${topScore ? `单一比分概率 ${formatPercent(topScore.probability)}，并非确定赛果` : "暂无比分矩阵"}</small></div>
      <div class="prediction-detail-team away"><span>客队</span><strong>${escapeHtml(data.awayTeam)}</strong><small>${data.awayLambda === null ? "预计进球暂无" : `预计进球 ${data.awayLambda.toFixed(2)}`}</small></div>
    </section>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>90 分钟结果</span><h3>胜平负概率</h3></div><small>概率越高，条形越长</small></div><div class="outcome-meter-grid">
      ${probabilityMeter("主胜", data.homeWin, data.homeWin === outcomeMax)}
      ${probabilityMeter("平局", data.draw, data.draw === outcomeMax)}
      ${probabilityMeter("客胜", data.awayWin, data.awayWin === outcomeMax)}
    </div></section>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>常用判断</span><h3>进球相关概率</h3></div></div><div class="prediction-market-grid">
      <div><span>双方都进球</span><strong>${data.btts === null ? "—" : formatPercent(data.btts)}</strong></div>
      <div><span>总进球大于 2.5</span><strong>${data.over25 === null ? "—" : formatPercent(data.over25)}</strong></div>
      <div><span>${escapeHtml(data.homeTeam)} 零封</span><strong>${data.homeCleanSheet === null ? "—" : formatPercent(data.homeCleanSheet)}</strong></div>
      <div><span>${escapeHtml(data.awayTeam)} 零封</span><strong>${data.awayCleanSheet === null ? "—" : formatPercent(data.awayCleanSheet)}</strong></div>
    </div></section>
    <section class="prediction-detail-section scoreline-exhaustive-section"><div class="section-heading"><div><span>比分分布</span><h3>概率不低于 0.1% 的比分</h3></div><small>显示 ${data.scorelines.filter((item) => item.probability >= 0.001).length} 项；更低概率结果归入矩阵尾部</small></div>
      ${
        data.scorelines.length === 0
          ? `<div class="empty-state compact"><strong>暂无比分数据</strong></div>`
          : `<div class="scoreline-exhaustive-grid threshold-scroll">${data.scorelines
              .filter((item) => item.probability >= 0.001)
              .map(
                (item) =>
                  `<div class="scoreline-exhaustive-row"><span>${item.rank}</span><strong>${escapeHtml(item.score)}</strong><i><b style="width:${Math.min(100, item.probability / Math.max(data.scorelines[0]?.probability ?? item.probability, 0.000001) * 100).toFixed(2)}%"></b></i><em>${formatPercent(item.probability)}</em><small>${item.cumulativeProbability === null ? "" : `累计 ${formatPercent(item.cumulativeProbability)}`}</small></div>`,
              )
              .join("")}</div>`
      }
      <div class="scoreline-threshold-note"><span>默认阈值：0.1%</span><span>完整矩阵、模型血缘与技术信息在详细设置中查看</span></div>
    </section>
    <section class="prediction-detail-section model-basis-section"><div class="section-heading"><div><span>模型依据</span><h3>本次推演使用的链路</h3></div></div><div class="model-basis-flow">
      <div><span>匹配方式</span><strong>${escapeHtml(data.routeSource)}</strong></div><i>→</i>
      <div><span>规则</span><strong>${escapeHtml(data.rulePackage)}</strong></div><i>→</i>
      <div><span>模型</span><strong>${escapeHtml(modelDisplayName(data.modelId))}</strong></div><i>→</i>
      <div><span>数据时点</span><strong>${escapeHtml(data.snapshotType)}</strong></div>
    </div></section>
    <section class="prediction-quality-strip"><div><span>概率合计检查</span><strong>${auditStatus === "PASS" ? "通过" : "需要检查"}</strong></div><div><span>比分矩阵覆盖</span><strong>${coverageStatus === "PASS" ? "通过" : "需要检查"}</strong></div><div><span>计算耗时</span><strong>${data.durationMs === null ? "—" : `${data.durationMs} 毫秒`}</strong></div><div><span>完成时间</span><strong>${data.createdAt ? new Date(data.createdAt).toLocaleString() : "本次运行"}</strong></div></section>
    ${predictionTechnicalDetails(data)}
  </div>`;
}

export function inlinePredictionResult(execution:unknown,mode:"formal"|"shadow"|"temporary"):string {
  const modeLabel=mode==="formal"?"正式推演":mode==="shadow"?"影子推演":"临时演练";
  const data=predictionViewData(execution),topScore=data.scorelines[0]??null;
  const probability=(value:number|null)=>value===null?"未记录":formatPercent(value);
  return `<header><h2>${modeLabel}完成</h2></header><div class="prediction-result-summary">
    <p class="prediction-result-identity">${escapeHtml(data.homeTeam)} vs ${escapeHtml(data.awayTeam)} · ${escapeHtml(data.snapshotType)} · ${escapeHtml(data.runId)}</p>
    <strong class="prediction-result-probabilities">主胜 ${probability(data.homeWin)} / 平局 ${probability(data.draw)} / 客胜 ${probability(data.awayWin)}</strong>
    <div class="prediction-result-score"><p>最可能比分 ${topScore?escapeHtml(topScore.score)+"（"+formatPercent(topScore.probability)+"）":"未记录"}</p><p>双方进球 ${probability(data.btts)} · 大于 2.5 球 ${probability(data.over25)}</p><p>${mode==="shadow"?"影子评估，不写入正式历史。":"单一比分并非确定结果；矩阵与输入审计可在详情核对。"}</p></div>
    <button class="secondary" data-action="show-last-result">查看完整结果</button>
  </div>`;
}

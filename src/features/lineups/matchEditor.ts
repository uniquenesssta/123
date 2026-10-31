import { refreshSearchableSelects } from "../../components/searchableSelect";
import type { MatchDraft } from "../../types";
import type { LineupContext, LineupSide } from "./lineupContext";
import { emptyPairedLineupSide } from "./lineupState";
export function createMatchEditor(c: LineupContext) {
    function inferCompetitionTeamScope(competitionId: string | null): "national" | "club" | "all" {
        const competition = c.ports.session.bootstrap?.data.competitions.find((item) => item.id === competitionId);
        if (!competition)
            return "all";
        const explicitScope = typeof competition.metadata?.scope === "string" ? competition.metadata.scope.toLowerCase() : "";
        if (explicitScope === "national" || explicitScope === "club")
            return explicitScope;
        const text = `${competition.name} ${competition.code}`.toLowerCase();
        if (/world cup|世界杯|euro|欧洲杯|nations|国家联赛|international|qualif|预选|copa america|美洲杯|asian cup|亚洲杯|afcon|非洲杯|gold cup|金杯|national/.test(text))
            return "national";
        if (/league|联赛|cup|杯|champions|libertadores|俱乐部|superliga|premier|bundesliga|serie|liga|ligue/.test(text))
            return "club";
        return "all";
    }
    function autoSelectMatchSeason(): void {
        const competitionId = c.ports.nullableValue("new-match-competition");
        const season = document.querySelector<HTMLSelectElement>("#new-match-season");
        const kickoffRaw = c.ports.nullableValue("new-match-kickoff");
        if (!competitionId || !season)
            return;
        const kickoffDate = kickoffRaw ? kickoffRaw.slice(0, 10) : null;
        const candidates = Array.from(season.options).filter((option) => option.value && option.dataset.competitionId === competitionId);
        const matched = kickoffDate
            ? candidates.find((option) => {
                const starts = option.dataset.startsOn || null;
                const ends = option.dataset.endsOn || null;
                return (!starts || starts <= kickoffDate) && (!ends || ends >= kickoffDate);
            })
            : null;
        if (!season.value || season.selectedOptions[0]?.hidden) {
            season.value = matched?.value ?? candidates[0]?.value ?? "";
        }
    }
    function updateCompetitionHierarchy(source: "scope" | "region" | "competition" | "init" = "init"): void {
        const scope = document.querySelector<HTMLSelectElement>("#new-match-competition-scope");
        const region = document.querySelector<HTMLSelectElement>("#new-match-competition-region");
        const competition = document.querySelector<HTMLSelectElement>("#new-match-competition");
        if (!scope || !region || !competition)
            return;
        const competitionOptions = Array.from(competition.options).filter((option) => option.value);
        const selectedScope = scope.value;
        if (source === "scope") {
            region.value = "";
            competition.value = "";
        }
        else if (source === "region") {
            competition.value = "";
        }
        const allowedRegions = new Set(competitionOptions
            .filter((option) => !selectedScope || option.dataset.scope === selectedScope)
            .map((option) => option.dataset.region ?? ""));
        for (const option of Array.from(region.options)) {
            if (!option.value)
                continue;
            const allowed = allowedRegions.has(option.value);
            option.hidden = !allowed;
            option.disabled = !allowed;
        }
        if (region.selectedOptions[0]?.disabled)
            region.value = "";
        region.disabled = !selectedScope;
        const regionPlaceholder = region.options[0];
        if (regionPlaceholder)
            regionPlaceholder.textContent = selectedScope ? "选择地区或足联" : "先选择参赛体系";
        const selectedRegion = region.value;
        competition.disabled = !selectedScope || !selectedRegion;
        const competitionPlaceholder = competition.options[0];
        if (competitionPlaceholder)
            competitionPlaceholder.textContent = selectedRegion ? "选择具体赛事" : "先选择地区或足联";
        for (const option of competitionOptions) {
            const allowed = (!selectedScope || option.dataset.scope === selectedScope) &&
                (!selectedRegion || option.dataset.region === selectedRegion);
            option.hidden = !allowed;
            option.disabled = !allowed;
        }
        if (competition.selectedOptions[0]?.disabled)
            competition.value = "";
        if (source !== "competition") {
            c.ports.filterSelectOptions("new-match-season", "competitionId", competition.value || null);
            c.ports.filterSelectOptions("new-match-stage", "competitionId", competition.value || null);
            autoSelectMatchSeason();
            filterMatchTeamOptions();
        }
        refreshSearchableSelects(c.ports.app);
    }
    function updateFormationHierarchy(side: LineupSide, source: "level1" | "level2" | "formation" | "init" = "init"): void {
        const level1 = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-level1`);
        const level2 = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-level2`);
        const formation = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-id`);
        const hiddenCode = document.querySelector<HTMLInputElement>(`#paired-${side}-formation`);
        if (!level1 || !level2 || !formation)
            return;
        if (source === "level1") {
            level2.value = "";
            formation.value = "";
        }
        else if (source === "level2") {
            formation.value = "";
        }
        for (const option of Array.from(level2.options)) {
            if (!option.value)
                continue;
            const allowed = !level1.value || option.dataset.level1 === level1.value;
            option.hidden = !allowed;
            option.disabled = !allowed;
        }
        if (level2.selectedOptions[0]?.disabled)
            level2.value = "";
        for (const option of Array.from(formation.options)) {
            if (!option.value)
                continue;
            const allowed = (!level1.value || option.dataset.level1 === level1.value) &&
                (!level2.value || option.dataset.level2 === level2.value);
            option.hidden = !allowed;
            option.disabled = !allowed;
        }
        if (formation.selectedOptions[0]?.disabled)
            formation.value = "";
        const selected = formation.selectedOptions[0];
        if (hiddenCode && selected?.value)
            hiddenCode.value = selected.dataset.code ?? "";
        if (source !== "init") {
            c.actions.capturePairedLineupFromDom();
            c.state.selectedMatchLineupChain = null;
        }
    }
    function filterMatchTeamOptions(): void {
        const competitionId = c.ports.nullableValue("new-match-competition");
        const seasonId = c.ports.nullableValue("new-match-season");
        const scopeSelect = document.querySelector<HTMLSelectElement>("#new-match-team-scope");
        const requested = scopeSelect?.value ?? "auto";
        const scope = requested === "auto" ? inferCompetitionTeamScope(competitionId) : requested;
        const registered = new Set((c.ports.references?.season_team_memberships ?? [])
            .filter((item) => item.season_id === seasonId && ["registered", "guest"].includes(item.registration_status))
            .map((item) => item.team_id));
        const restrictMembership = Boolean(seasonId && registered.size > 0);
        for (const selectId of ["new-match-home-team", "new-match-away-team"]) {
            const select = document.querySelector<HTMLSelectElement>(`#${selectId}`);
            if (!select)
                continue;
            for (const option of Array.from(select.options)) {
                if (!option.value) {
                    option.hidden = false;
                    continue;
                }
                const typeMatches = scope === "all" || option.dataset.teamType === scope;
                const memberMatches = !restrictMembership || registered.has(option.value);
                option.hidden = !option.selected && !(typeMatches && memberMatches);
            }
            if (select.selectedOptions[0]?.hidden)
                select.value = "";
        }
        const note = document.querySelector<HTMLElement>("#match-team-filter-note");
        if (note) {
            note.textContent = restrictMembership
                ? `当前赛季已注册 ${registered.size} 支球队，主客队仅从注册名单选择。`
                : scope === "national"
                    ? "当前赛事按国家队体系筛选。"
                    : scope === "club"
                        ? "当前赛事按俱乐部体系筛选。"
                        : "当前赛事未限定球队体系。";
        }
    }
    function filterManagedMatchList(search: string): void {
        const normalized = search.trim().toLowerCase();
        for (const item of Array.from(document.querySelectorAll<HTMLElement>(".match-list-item"))) {
            item.hidden = normalized.length > 0 && !(item.textContent ?? "").toLowerCase().includes(normalized);
        }
    }
    let savingMatch = false;
    async function createMatch(): Promise<void> {
        if (!c.active)
            throw new Error("比赛工作区已关闭");
        if (savingMatch)
            return;
        autoSelectMatchSeason();
        const competitionId = c.ports.nullableValue("new-match-competition");
        const seasonId = c.ports.nullableValue("new-match-season");
        const homeTeamId = c.ports.nullableValue("new-match-home-team");
        const awayTeamId = c.ports.nullableValue("new-match-away-team");
        const kickoff = c.ports.localDateTimeToIso(c.ports.nullableValue("new-match-kickoff"));
        if (!competitionId)
            throw new Error("请先选择赛事；缺少赛事时可跳转赛事设置补充后返回");
        if (!homeTeamId || !awayTeamId || !kickoff) {
            throw new Error("请选择主客队并填写开球时间");
        }
        if (homeTeamId === awayTeamId)
            throw new Error("主队和客队不能相同");
        const existingExternalKey = c.ports.nullableValue("managed-match-external-key");
        const draft: MatchDraft = {
            external_key: existingExternalKey ?? `manual:${competitionId}:${homeTeamId}:${awayTeamId}:${kickoff}`,
            competition_id: competitionId,
            season_id: seasonId,
            stage_id: c.ports.nullableValue("new-match-stage"),
            round_id: c.ports.nullableValue("new-match-round"),
            home_team_id: homeTeamId,
            away_team_id: awayTeamId,
            kickoff_time: kickoff,
            status: c.ports.value("new-match-status") as MatchDraft["status"],
            venue: c.ports.nullableValue("new-match-venue"),
            metadata: { entry_mode: "match_center_hierarchy" },
        };
        const selected = c.state.selectedManagedMatchId, page = c.ports.navigation.page;
        const identity = () => JSON.stringify(["new-match-competition", "new-match-season", "new-match-home-team", "new-match-away-team", "new-match-kickoff", "managed-match-external-key", "new-match-stage", "new-match-round", "new-match-status", "new-match-venue"].map(id => c.ports.nullableValue(id)));
        const submittedIdentity = identity();
        const current = () => c.active && selected === c.state.selectedManagedMatchId && page === c.ports.navigation.page && submittedIdentity === identity();
        savingMatch = true;
        try {
            const saved = await c.ports.runBusy(() => c.ports.api.createMatch(draft));
            if (!current()) {
                c.ports.toast("原比赛已保存，请刷新对应目录查看", "success");
                return;
            }
            await c.ports.session.refresh();
            if (!current())
                return;
            const references = await c.ports.api.playerCatalogReferenceData();
            if (!current())
                return;
            const records = await c.ports.api.listLineups(null, 100);
            if (!current())
                return;
            c.ports.setReferences(references);
            c.state.lineupRecords = records;
            c.state.selectedManagedMatchId = saved.id;
            c.ports.render({ preserveForm: true });
            c.ports.toast(existingExternalKey ? "比赛修改已保存" : "比赛已创建", "success");
        }
        finally {
            savingMatch = false;
        }
    }
    function requestRemoveLineupHistory(lineupId: string, lineupLabel: string): void {
        if (!lineupId)
            throw new Error("缺少阵容版本编号");
        const matchId = c.state.pairedLineupBuilder.match_id, snapshot = c.state.pairedLineupBuilder.snapshot_type, page = c.ports.navigation.page;
        const current = () => c.active && page === c.ports.navigation.page && matchId === c.state.pairedLineupBuilder.match_id && snapshot === c.state.pairedLineupBuilder.snapshot_type;
        c.ports.showConfirmation("删除阵容历史", "未被正式推演、快照或球员贡献引用的版本会永久删除；已被引用的版本会归档并从历史列表隐藏，模型血缘继续保留。", [
            ["阵容", lineupLabel],
            ["活动版本", "删除后自动恢复上一有效版本（如存在）"],
        ], "确认处理阵容版本", async () => {
            if (!current())
                throw new Error("阵容历史上下文已变化，请重新确认");
            const result = await c.ports.runBusy(() => c.ports.api.removeLineupHistory(lineupId, "用户从阵容历史删除"));
            if (!current())
                return;
            const records = await c.ports.api.listLineups(null, 100);
            if (!current())
                return;
            const chain = matchId ? await c.ports.api.readMatchLineupChain(matchId, snapshot) : null;
            if (!current())
                return;
            c.state.lineupRecords = records;
            c.state.selectedMatchLineupChain = chain;
            c.ports.render();
            c.ports.toast(result.removal_mode === "deleted"
                ? "未引用阵容版本已永久删除"
                : "已引用阵容版本已归档并隐藏", "success");
        });
    }
    function requestDeleteMatch(matchId: string, matchLabel: string): void {
        const selected = c.state.selectedManagedMatchId, page = c.ports.navigation.page;
        c.ports.showConfirmation("删除比赛", "已有模型运行、P4研究、冻结或正式赛后结算引用的比赛必须保留。未受保护比赛可永久删除其比赛、阵容、赛果和普通复盘；服务端会再次核对历史保护。", [
            ["比赛", matchLabel],
            ["保护", "模型运行、P4研究、冻结或正式结算存在时禁止永久删除"],
            ["保留", "已产生模型运行的比赛及其不可变历史血缘"],
        ], "永久删除比赛", async () => {
            if (!c.active || selected !== c.state.selectedManagedMatchId || page !== c.ports.navigation.page)
                throw new Error("当前比赛上下文已变化，请重新确认删除");
            await c.ports.runBusy(() => c.ports.api.deleteMatch(matchId));
            if (!c.active || selected !== c.state.selectedManagedMatchId || page !== c.ports.navigation.page) {
                c.ports.toast("原比赛删除已完成，请刷新对应目录", "success");
                return;
            }
            c.state.lineupBuilderForm = { ...c.state.lineupBuilderForm, match_id: "", team_id: "" };
            c.state.lineupBuilderPlayers = [];
            if (c.state.pairedLineupBuilder.match_id === matchId) {
                c.state.pairedLineupBuilder = {
                    ...c.state.pairedLineupBuilder,
                    match_id: "",
                    home: emptyPairedLineupSide(),
                    away: emptyPairedLineupSide(),
                };
            }
            if (c.state.selectedManagedMatchId === matchId)
                c.state.selectedManagedMatchId = null;
            await c.ports.refreshBootstrapAndCatalog();
            if (!c.active || page !== c.ports.navigation.page)
                return;
            const records = await c.ports.api.listLineups(null, 100);
            if (!c.active || page !== c.ports.navigation.page)
                return;
            c.state.lineupRecords = records;
            c.ports.render();
            c.ports.toast("比赛及关联阵容已删除", "success");
        });
    }
    return { inferCompetitionTeamScope, autoSelectMatchSeason, updateCompetitionHierarchy, updateFormationHierarchy, filterMatchTeamOptions, filterManagedMatchList, createMatch, requestRemoveLineupHistory, requestDeleteMatch };
}

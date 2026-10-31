import { positionLabel } from "../../components/footballText";
import { escapeHtml } from "../../components/format";
import type { LineupContext } from "./lineupContext";
export function createPresetPositions(c: LineupContext) {
    const TACTICAL_ROLE_OPTIONS: Readonly<Record<string, readonly string[]>> = {
        GK: ["门线型门将", "出击型门将", "清道夫门将"],
        CB: ["盯人中卫", "出球中卫", "拖后中卫"],
        FB: ["防守型边后卫", "进攻型边后卫", "内收边后卫"],
        WB: ["翼卫", "进攻型翼卫", "防守型翼卫"],
        DM: ["单后腰", "防守型后腰", "组织型后腰", "抢球型中场"],
        CM: ["全能中场", "组织核心", "控球中场", "抢球型中场"],
        AM: ["前场组织核心", "组织核心", "影锋", "边路组织者"],
        W: ["边锋", "内切边锋", "边路组织者"],
        SS: ["影锋", "前场组织核心", "第二前锋"],
        ST: ["抢点中锋", "支点中锋", "全能前锋", "伪九号", "反击前锋"],
    };
    function presetContextLabel(value: string): string {
        return ({
            general: "通用",
            league: "联赛常规",
            cup: "杯赛轮换",
            attacking: "进攻方案",
            defensive: "防守方案",
            rotation: "轮换方案",
            temporary: "临时方案",
        } as Record<string, string>)[value] ?? value;
    }
    function formationSlotCodes(formationId: string | null): string[] {
        if (!formationId)
            return [];
        const formation = c.ports.formationCatalog.find((item) => item.id === formationId);
        if (!formation || !Array.isArray(formation.slot_definition))
            return [];
        return [...new Set(formation.slot_definition.filter((item): item is string => typeof item === "string" && item.trim().length > 0).map((item) => item.trim().toUpperCase()))];
    }
    function positionGroupKey(positionCode: string | null): keyof typeof TACTICAL_ROLE_OPTIONS {
        const code = (positionCode ?? "").trim().toUpperCase();
        if (code === "GK")
            return "GK";
        if (["CB", "LCB", "RCB", "SW"].includes(code))
            return "CB";
        if (["LB", "RB"].includes(code))
            return "FB";
        if (["LWB", "RWB"].includes(code))
            return "WB";
        if (["DM", "CDM", "LDM", "RDM"].includes(code))
            return "DM";
        if (["CM", "LCM", "RCM", "LM", "RM"].includes(code))
            return "CM";
        if (["AM", "CAM", "LAM", "RAM"].includes(code))
            return "AM";
        if (["LW", "RW", "LF", "RF"].includes(code))
            return "W";
        if (code === "SS")
            return "SS";
        return "ST";
    }
    function tacticalRoleOptions(positionCode: string | null, inheritedRole: string | null, explicitRole: string | null): string {
        const roles = new Set<string>();
        if (positionCode?.trim()) {
            for (const role of TACTICAL_ROLE_OPTIONS[positionGroupKey(positionCode)] ?? [])
                roles.add(role);
        }
        if (inheritedRole?.trim())
            roles.add(inheritedRole.trim());
        if (explicitRole?.trim())
            roles.add(explicitRole.trim());
        const inheritLabel = inheritedRole?.trim()
            ? `自动继承：${inheritedRole.trim()}`
            : "自动继承球员资料角色";
        return `<option value="" ${explicitRole ? "" : "selected"}>${escapeHtml(inheritLabel)}</option>${[...roles].map((role) => `<option value="${escapeHtml(role)}" ${role === explicitRole ? "selected" : ""}>${escapeHtml(role)}</option>`).join("")}`;
    }
    function positionOptionGroups(formationId: string | null, selected: string | null): string {
        const formationSlots = formationSlotCodes(formationId);
        const regularPositions = c.ports.references?.positions ?? [];
        const known = new Set<string>();
        const renderOption = (code: string, label: string): string => {
            const normalized = code.trim().toUpperCase();
            if (!normalized || known.has(normalized))
                return "";
            known.add(normalized);
            return `<option value="${escapeHtml(normalized)}" ${normalized === selected?.trim().toUpperCase() ? "selected" : ""}>${escapeHtml(label)}（${escapeHtml(normalized)}）</option>`;
        };
        const slotOptions = formationSlots.map((code) => renderOption(code, positionLabel(code))).join("");
        const regularOptions = regularPositions.map((item) => renderOption(item.code, item.name || positionLabel(item.code))).join("");
        const selectedCode = selected?.trim().toUpperCase() ?? "";
        const currentOption = selectedCode && !known.has(selectedCode)
            ? `<optgroup label="当前保存值">${renderOption(selectedCode, positionLabel(selectedCode))}</optgroup>`
            : "";
        return `<option value="">请选择战术位置</option>${slotOptions ? `<optgroup label="当前阵型槽位">${slotOptions}</optgroup>` : ""}${regularOptions ? `<optgroup label="常规位置">${regularOptions}</optgroup>` : ""}${currentOption}`;
    }
    function presetPositionCompatibility(playerPosition: string, slot: string): number {
        const player = playerPosition.trim().toUpperCase();
        const target = slot.trim().toUpperCase();
        if (!player)
            return 0;
        if (player === target)
            return 100;
        const aliases: Record<string, readonly string[]> = {
            GK: ["GK"],
            CB: ["CB", "LCB", "RCB", "SW"],
            LB: ["LB", "LWB"],
            RB: ["RB", "RWB"],
            LWB: ["LWB", "LB", "LM"],
            RWB: ["RWB", "RB", "RM"],
            DM: ["DM", "CDM", "LDM", "RDM", "CM"],
            CDM: ["DM", "CDM", "LDM", "RDM"],
            CM: ["CM", "LCM", "RCM", "DM", "AM"],
            AM: ["AM", "CAM", "LAM", "RAM", "CM", "SS"],
            CAM: ["CAM", "AM", "LAM", "RAM", "SS"],
            LM: ["LM", "LW", "LWB", "LCM"],
            RM: ["RM", "RW", "RWB", "RCM"],
            LW: ["LW", "LM", "LAM", "LF"],
            RW: ["RW", "RM", "RAM", "RF"],
            SS: ["SS", "AM", "CAM", "CF", "ST"],
            CF: ["CF", "ST", "SS", "LST", "RST"],
            ST: ["ST", "CF", "LST", "RST", "SS"],
        };
        const playerAliases = aliases[player] ?? [player];
        const targetBase = target.replace(/^[LR](?=(CB|DM|CM|AM|ST)$)/, "");
        if (playerAliases.includes(target))
            return 90;
        if (playerAliases.includes(targetBase))
            return 82;
        if (positionGroupKey(player) === positionGroupKey(target))
            return 60;
        return 10;
    }
    function setNativeSelectOptions(select: HTMLSelectElement, optionsHtml: string, preferredValue: string | null): void {
        select.innerHTML = optionsHtml;
        if (preferredValue && Array.from(select.options).some((option) => option.value === preferredValue)) {
            select.value = preferredValue;
        }
    }
    function refreshPresetTacticalRoleSelect(row: HTMLElement): void {
        const positionSelect = row.querySelector<HTMLSelectElement>(".preset-member-position");
        const roleSelect = row.querySelector<HTMLSelectElement>(".preset-member-tactical-role");
        if (!positionSelect || !roleSelect)
            return;
        const current = roleSelect.value || null;
        const inherited = row.dataset.playerInheritedRole || null;
        setNativeSelectOptions(roleSelect, tacticalRoleOptions(positionSelect.value || null, inherited, current), current);
    }
    function syncPresetMemberRowState(row: HTMLElement): void {
        const enabled = row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked ?? false;
        const role = row.querySelector<HTMLSelectElement>(".preset-member-role")?.value ?? "substitute";
        row.classList.toggle("excluded", !enabled);
        row.classList.toggle("starter", enabled && role === "starter");
        row.classList.toggle("substitute", enabled && role !== "starter");
        row.querySelectorAll<HTMLInputElement | HTMLSelectElement>("select, input[type='radio']").forEach((control) => {
            control.disabled = !enabled;
        });
        const captain = row.querySelector<HTMLInputElement>(".preset-member-captain input");
        if (!enabled && captain)
            captain.checked = false;
    }
    function syncPresetEditorRows(): void {
        document.querySelectorAll<HTMLElement>("[data-preset-player-id]").forEach((row) => {
            syncPresetMemberRowState(row);
            refreshPresetTacticalRoleSelect(row);
        });
    }
    function presetEditorFormationId(): string | null {
        return document.querySelector<HTMLSelectElement>("#lineup-preset-formation")?.value || null;
    }
    function refreshPresetPositionOptions(autoAssign = false): void {
        const formationId = presetEditorFormationId();
        const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
        for (const row of rows) {
            const select = row.querySelector<HTMLSelectElement>(".preset-member-position");
            if (!select)
                continue;
            const current = select.value || row.dataset.playerDefaultPosition || null;
            setNativeSelectOptions(select, positionOptionGroups(formationId, current), current);
        }
        if (autoAssign)
            assignPresetFormationSlots(false);
        rows.forEach(refreshPresetTacticalRoleSelect);
        updatePresetEditorSummary();
    }
    function assignPresetFormationSlots(showFeedback = true): void {
        const formationId = presetEditorFormationId();
        const slots = formationSlotCodes(formationId);
        if (slots.length !== 11) {
            if (showFeedback)
                c.ports.toast("请先选择一个包含 11 个战术位置的标准阵型", "error");
            return;
        }
        const starterRows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"))
            .filter((row) => row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked)
            .filter((row) => row.querySelector<HTMLSelectElement>(".preset-member-role")?.value === "starter");
        if (starterRows.length !== 11) {
            if (showFeedback)
                c.ports.toast(`自动分配需要恰好 11 名首发，当前为 ${starterRows.length} 名`, "error");
            return;
        }
        const availableRows = new Set(starterRows);
        for (const slot of slots) {
            const best = [...availableRows].sort((left, right) => {
                const leftScore = presetPositionCompatibility(left.dataset.playerDefaultPosition ?? "", slot);
                const rightScore = presetPositionCompatibility(right.dataset.playerDefaultPosition ?? "", slot);
                return rightScore - leftScore;
            })[0];
            if (!best)
                continue;
            availableRows.delete(best);
            const select = best.querySelector<HTMLSelectElement>(".preset-member-position");
            if (select) {
                setNativeSelectOptions(select, positionOptionGroups(formationId, slot), slot);
                select.value = slot;
                refreshPresetTacticalRoleSelect(best);
            }
        }
        updatePresetEditorSummary();
        if (showFeedback)
            c.ports.toast("已按阵型和球员登记位置完成首发槽位分配", "success");
    }
    function updatePresetEditorSummary(): void {
        const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
        const enabled = rows.filter((row) => row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked);
        const starters = enabled.filter((row) => row.querySelector<HTMLSelectElement>(".preset-member-role")?.value === "starter");
        const substitutes = enabled.length - starters.length;
        const starterPositions = starters.map((row) => row.querySelector<HTMLSelectElement>(".preset-member-position")?.value ?? "").filter(Boolean);
        const duplicatePositions = starterPositions.filter((value, index, values) => values.indexOf(value) !== index);
        const formationSlots = formationSlotCodes(presetEditorFormationId());
        const missingSlots = formationSlots.filter((slot) => !starterPositions.includes(slot));
        const missingPositionCount = starters.length - starterPositions.length;
        const ready = starters.length === 11
            && missingPositionCount === 0
            && duplicatePositions.length === 0
            && (formationSlots.length === 0 || missingSlots.length === 0);
        const summary = document.querySelector<HTMLElement>("#lineup-preset-member-summary");
        if (summary) {
            summary.textContent = `已选 ${enabled.length} 人 · 首发 ${starters.length}/11 · 替补 ${substitutes}`;
            summary.classList.toggle("ready", ready);
            summary.classList.toggle("warning", !ready);
        }
        const note = document.querySelector<HTMLElement>("#lineup-preset-validation-note");
        if (!note)
            return;
        const issues: string[] = [];
        if (starters.length !== 11)
            issues.push(`首发应为 11 人，当前 ${starters.length} 人`);
        if (missingPositionCount > 0)
            issues.push(`${missingPositionCount} 名首发尚未选择战术位置`);
        if (duplicatePositions.length > 0)
            issues.push(`首发战术位置重复：${[...new Set(duplicatePositions)].join("、")}`);
        if (formationSlots.length > 0 && missingSlots.length > 0)
            issues.push(`阵型槽位未覆盖：${missingSlots.join("、")}`);
        note.classList.toggle("ready", issues.length === 0);
        note.classList.toggle("warning", issues.length > 0);
        note.textContent = issues.length ? issues.join("；") : "阵容结构有效，可以保存。";
    }
    return { presetContextLabel, formationSlotCodes, positionGroupKey, tacticalRoleOptions, positionOptionGroups, presetPositionCompatibility, setNativeSelectOptions, refreshPresetTacticalRoleSelect, syncPresetMemberRowState, syncPresetEditorRows, presetEditorFormationId, refreshPresetPositionOptions, assignPresetFormationSlots, updatePresetEditorSummary };
}

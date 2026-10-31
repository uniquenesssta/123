import type { SpreadsheetImportMode } from "../../types";
import type { LineupContext } from "./lineupContext";
import type { MatchImports } from "./matchImports";
export function createMatchExchange(c: LineupContext, imports: MatchImports) {
    function selectedExchangeMatchId(): string { const id = c.ports.nullableValue("exchange-match-id"); if (!id)
        throw new Error("请先选择比赛"); return id; }
    async function exportMatchTemplate(): Promise<void> { const path = await c.ports.files.chooseExcelExportFile("比赛与阵容输入模板.xlsx"); if (!path || !c.active)
        return; const result = await c.ports.runBusy(() => c.ports.api.exportMatchLineupTemplate(path)); c.ports.toast("比赛模板已导出：" + result.output_path, "success"); }
    async function exportMatchData(): Promise<void> { const id = selectedExchangeMatchId(), path = await c.ports.files.chooseExcelExportFile("比赛与阵容导出.xlsx"); if (!path || !c.active)
        return; const result = await c.ports.runBusy(() => c.ports.api.exportMatchLineupData(path, id)); c.ports.toast("已导出 " + result.lineup_count + " 份阵容和 " + result.player_count + " 名球员", "success"); }
    async function exportAiMatchPackage(): Promise<void> { const id = selectedExchangeMatchId(), path = await c.ports.files.chooseZipExportFile("比赛推演分析包.zip"); if (!path || !c.active)
        return; const result = await c.ports.runBusy(() => c.ports.api.exportAiMatchPackage(path, id)); c.ports.toast("智能分析资料已导出，包含 " + result.player_count + " 名球员信息", "success"); }
    async function previewMatchImport(aiPackage: boolean): Promise<void> {
        const mode = c.ports.value("match-import-mode") as SpreadsheetImportMode;
        if (!await imports.read(aiPackage, mode))
            return;
        const preview = imports.preview;
        if (!preview)
            return;
        c.ports.render();
        const blocking = preview.counts.conflict + preview.counts.error;
        c.ports.toast(blocking ? "预检完成：" + blocking + " 条需要处理" : "比赛与阵容预检通过", blocking ? "error" : "success");
    }
    async function resolveMatchImportConflict(rowId: string, entityId: string | null, skip: boolean): Promise<void> { if (await imports.resolve({ row_id: rowId, selected_entity_id: entityId, skip }))
        c.ports.render(); }
    async function commitMatchImport(): Promise<void> {
        const saved = await imports.commit();
        if (!saved?.current) {
            if (saved)
                c.ports.toast("原比赛批次提交已返回，请在原批次核对", "normal");
            return;
        }
        const preview = imports.preview, page = c.ports.navigation.page;
        await c.ports.refreshBootstrapAndCatalog();
        if (!c.active || page !== c.ports.navigation.page || preview !== imports.preview)
            return;
        const records = await c.ports.api.listLineups(null, 100);
        if (!c.active || page !== c.ports.navigation.page || preview !== imports.preview)
            return;
        c.state.lineupRecords = records;
        c.ports.render();
        const result = saved.result, partial = result.error_count > 0;
        c.ports.toast((partial ? "比赛与阵容部分导入：" : "比赛与阵容导入完成：") + "新增 " + result.inserted_count + "，更新 " + result.updated_count + "，错误 " + result.error_count, partial ? "error" : "success");
    }
    return { selectedExchangeMatchId, exportMatchTemplate, exportMatchData, exportAiMatchPackage, previewMatchImport, resolveMatchImportConflict, commitMatchImport };
}

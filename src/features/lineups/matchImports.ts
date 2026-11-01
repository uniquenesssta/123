import type { SpreadsheetImportMode } from "../../types";
import { SpreadsheetBatch, type ImportRecovery } from "../../app/imports/spreadsheetBatch";
import type { LineupApi, LineupFiles } from "./lineupPorts";
interface ImportPorts { api: LineupApi; files: LineupFiles; capture(): () => boolean; captureView?(): () => boolean; changed?(): void; recovery?: ImportRecovery; busy<T>(operation: () => Promise<T>): Promise<T> }
/** Match XLSX and AI packages share one staging/recovery owner. */
export class MatchImports extends SpreadsheetBatch {
  constructor(private readonly adapters: ImportPorts) {
    super({ ...adapters, label:"比赛", choose:()=>adapters.files.chooseExcelImportFile(),
      preview:(path,mode)=>adapters.api.previewMatchLineupImport(path,mode),
      resolve:(id,resolution)=>adapters.api.resolveMatchLineupImportConflict(id,resolution),
      commit:id=>adapters.api.commitMatchLineupImport(id), read:id=>adapters.api.readMatchLineupImportPreview(id) });
  }
  read(aiPackage: boolean, mode: SpreadsheetImportMode): Promise<boolean> {
    return aiPackage ? this.load(mode,()=>this.adapters.files.chooseZipImportFile(),(path,mode)=>this.adapters.api.previewAiMatchPackage(path,mode)) : this.load(mode);
  }
}

import type { SpreadsheetImportMode } from "../../types";
import { SpreadsheetBatch, type ImportRecovery } from "../../app/imports/spreadsheetBatch";
import type { PlayerApi } from "./playerPorts";
interface ImportPorts { api: PlayerApi; capture(): () => boolean; captureView?(): () => boolean; changed?(): void; recovery?: ImportRecovery; busy<T>(operation: () => Promise<T>): Promise<T> }
/** Same player batch owner consumed by Players and Workbooks. */
export class PlayerImports extends SpreadsheetBatch {
  constructor(ports: ImportPorts) {
    super({ ...ports, label:"球员", choose:()=>ports.api.chooseExcelImportFile(),
      preview:(path,mode)=>ports.api.previewPlayerCatalogImport(path,mode),
      resolve:(id,resolution)=>ports.api.resolvePlayerCatalogImportConflict(id,resolution),
      commit:id=>ports.api.commitPlayerCatalogImport(id), read:id=>ports.api.readPlayerCatalogImportPreview(id) });
  }
  read(mode: SpreadsheetImportMode): Promise<boolean> { return this.load(mode); }
}

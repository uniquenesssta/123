import type {IssueLogEntry} from '../../types';

export type LogSection='overview'|'processing'|'records';
export interface LogRecovery {section:LogSection;expanded:string[];unknownClear:boolean;operationId:string|null}
export interface LogsApi {
  listIssueLogs(limit?:number):Promise<IssueLogEntry[]>;
  chooseIssueLogExportFile(defaultPath:string):Promise<string|null>;
  exportIssueLogs(outputPath:string,expectedEntries?:IssueLogEntry[]):Promise<void>;
  clearIssueLogs(expectedEntries?:IssueLogEntry[]):Promise<void>;
}
export interface LogsPorts {
  api:LogsApi;
  view():()=>boolean;
  changed():void;
  busy<T>(run:()=>Promise<T>):Promise<T>;
  storage:{read():LogRecovery;save(value:LogRecovery):void};
}
export interface LogState {
  entries:IssueLogEntry[]|null;loading:boolean;readError:string|null;readAt:string|null;
  section:LogSection;expanded:Set<string>;operation:'choosing'|'exporting'|'checking'|'clearing'|null;
  status:string|null;unknownClear:boolean;checkedUnknown:boolean;operationId:string|null;storageError:string|null;
}
export function logSnapshot(entries:IssueLogEntry[]):string {return JSON.stringify(entries);}

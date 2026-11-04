import type {ApiWorkspacePreset,ApiWorkspaceSessionDetail,ApiWorkspaceSessionRecord,OpenAiProfilesState,PlayerCatalogReferenceData,SendApiWorkspaceCommand} from '../../types';
export interface AiDraft {message:string;profileId:string;presetKey:string;matchId:string|null;entityType:'team'|'player'|null;entityId:string|null;entityLabel:string|null;includeContext:boolean}
export interface AiReceipt {id:string;origin:string;sessionId:string|null;startedAt:string;phase:'sending'|'cancelling'|'completed'|'cancelled'|'unknown';error:string|null;released:boolean;checked:boolean}
export interface AiSavedState {selected:string;drafts:Record<string,AiDraft>;receipt:AiReceipt|null;archives:Record<string,'pending'|'unknown'>;search:string}
export interface AiStorageScope {read():AiSavedState|null;save(value:AiSavedState):void}
export interface AiStorage {capture():AiStorageScope}
export interface AiWorkspacePorts {
 api:{listOpenAiProfiles():Promise<OpenAiProfilesState>;listApiWorkspacePresets():Promise<ApiWorkspacePreset[]>;listApiWorkspaceSessions(limit:number):Promise<ApiWorkspaceSessionRecord[]>;playerCatalogReferenceData():Promise<PlayerCatalogReferenceData>;readApiWorkspaceSession(id:string):Promise<ApiWorkspaceSessionDetail>;sendApiWorkspaceMessage(command:SendApiWorkspaceCommand):Promise<ApiWorkspaceSessionDetail>;cancelApiWorkspaceRequest(id:string):Promise<boolean>;archiveApiWorkspaceSession(id:string):Promise<void>};
 storage:AiStorage;scope():()=>boolean;view():()=>boolean;changed():void;formatError(error:unknown):string;
}
export const emptyAiDraft=():AiDraft=>({message:'',profileId:'',presetKey:'plain_chat',matchId:null,entityType:null,entityId:null,entityLabel:null,includeContext:false});

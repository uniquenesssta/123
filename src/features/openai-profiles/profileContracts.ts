import type {OpenAiApiExampleParseResult,OpenAiApiProtocol,OpenAiProfileDraft,OpenAiProfileSummary,OpenAiProfilesState,OpenAiProfileTestResult} from '../../types';
export type ProfileDraft=Omit<OpenAiProfileDraft,'api_key'>;
export type ProfileSection='profiles'|'request'|'models'|'tests'|'example';
export type ProfileOperationKind='save'|'test'|'activate'|'clear'|'delete';
export interface ProfileOperation {id:string;kind:ProfileOperationKind;phase:'pending'|'unknown';profileId:string|null;checked:boolean;released:boolean;message:string}
export interface ProfileEntry {draft:ProfileDraft;revision:number;dirty:boolean}
export interface ProfileSavedState {selected:string;drafts:Record<string,ProfileEntry>;operations:Record<string,ProfileOperation>;section:ProfileSection}
export interface ProfilesPorts {
 api:{listOpenAiProfiles():Promise<OpenAiProfilesState>;parseOpenAiApiExample(example:string,protocol:OpenAiApiProtocol|null):Promise<OpenAiApiExampleParseResult>;saveOpenAiProfile(draft:OpenAiProfileDraft):Promise<OpenAiProfileSummary>;testOpenAiProfile(id:string):Promise<OpenAiProfileTestResult>;setActiveOpenAiProfile(id:string):Promise<OpenAiProfilesState>;clearOpenAiProfileKey(id:string):Promise<OpenAiProfilesState>;deleteOpenAiProfile(id:string):Promise<OpenAiProfilesState>};
 storage:{read():ProfileSavedState|null;save(value:ProfileSavedState):void};view():()=>boolean;changed():void;formatError(error:unknown):string;
}

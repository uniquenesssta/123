import type {PostmatchOverview} from '../../types';
import type {PostmatchPorts,PostmatchRuntime} from './postmatchPorts';
import {validateOverview} from './postmatchIdentity';
import {SettlementWorkflow} from './settlementWorkflow';
import {EvidenceWorkflow} from './evidenceWorkflow';
import {MonitoringWorkflow} from './monitoringWorkflow';
/** Owns postmatch reads and composes settlement, evidence, and monitoring without a page-level ledger. */
export class PostmatchFeature {
  private data:PostmatchOverview|null=null;
  private valid=()=>false;
  private viewRevision=0;private interactionRevision=0;
  private readRevision=0;
  private filter='';
  private feedback:string|null=null;
  private reading=false;
  readonly settlement:SettlementWorkflow;
  readonly evidence:EvidenceWorkflow;
  readonly monitoring:MonitoringWorkflow;
  constructor(private readonly ports:PostmatchPorts){
    const runtime:PostmatchRuntime={...ports,enter:()=>this.enter(),current:()=>this.capture(),interactionChanged:()=>this.interactionChanged(),overview:()=>this.overview,refresh:()=>this.refresh()};
    this.settlement=new SettlementWorkflow(runtime);
    this.evidence=new EvidenceWorkflow(runtime,()=>this.filter);
    this.monitoring=new MonitoringWorkflow(runtime);
  }
  private enter():void {
    if(this.valid())return;
    this.valid=this.ports.scope();this.data=null;this.filter='';this.feedback=null;this.reading=false;this.readRevision++;
    this.settlement.reset();this.evidence.reset();this.monitoring.reset();
  }
  get overview():PostmatchOverview|null{this.enter();return this.data;}
  get error():string|null{this.enter();return this.feedback;}
  get loading():boolean{this.enter();return this.reading;}
  get settlementFilter():string{this.enter();return this.filter;}
  capture(includeInteraction=true):()=>boolean {const scope=this.ports.scope(),view=this.ports.view(),revision=this.viewRevision,interaction=this.interactionRevision;return()=>scope()&&view()&&revision===this.viewRevision&&(!includeInteraction||interaction===this.interactionRevision);}
  interactionChanged():void{this.interactionRevision++;}
  navigationChanged():void{this.viewRevision++;this.interactionChanged();this.readRevision++;this.reading=false;}
  selectSettlement(id:string):void {
    this.enter();if(id&&!this.data?.settlements.some(row=>row.id===id))throw Error('结算不属于当前已载入目录');
    this.filter=id;this.interactionChanged();this.ports.changed();
  }
  async refresh():Promise<boolean>{
    this.enter();const serial=++this.readRevision,current=this.capture(false);this.reading=true;this.feedback=null;this.ports.changed();
    try{const value=await this.ports.api.postmatchOverview(500);if(!current()||serial!==this.readRevision)return false;validateOverview(value);this.data=value;return true;}
    catch(error){if(current()&&serial===this.readRevision)this.feedback='赛后读取失败，保留上次数据；不能当作没有记录。 '+this.ports.formatError(error);return false;}
    finally{if(current()&&serial===this.readRevision){this.reading=false;this.ports.changed();}}
  }
}

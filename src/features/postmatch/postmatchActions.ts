import type {EvidenceVerdict} from '../../types';
import {escapeHtml as html} from '../../components/format';
import {buttonView} from '../../ui/primitives/button';
import {textInputView} from '../../ui/primitives/text-input';
import type {PostmatchFeature} from './postmatchFeature';
import type {SettlementTarget} from './postmatchPorts';
export interface PostmatchActionPorts {
  review(id?:string):{target:SettlementTarget;current:()=>boolean};
  openSettlement():void;refreshReview():Promise<void>;
  confirm(title:string,description:string,facts:Array<[string,string]>,label:string,action:()=>Promise<void>):void;
  showHtml(title:string,subtitle:string,body:string,footer?:string):void;
  showResult(title:string,value:unknown):void;close():void;notify(message:string):void;
}
const labels:Record<EvidenceVerdict,string>={correct:'正确',partial:'部分正确',incorrect:'错误',not_verifiable:'无法验证'};
const button=(label:string,action:string,attributes:Record<string,string>={},primary=false)=>buttonView({label,variant:primary?'primary':'secondary',attributes:{'data-action':action,...attributes}});
/** UI intentions and protected confirmation content; all data operations remain workflow-owned. */
export class PostmatchActions {
  private evidenceDialog:{id:string;verdict:EvidenceVerdict;token:string;current:()=>boolean}|null=null;
  constructor(private readonly feature:PostmatchFeature,private readonly ports:PostmatchActionPorts){}
  async inspect(id:string):Promise<void>{this.ports.openSettlement();const target=this.ports.review(id);await this.feature.settlement.inspect(target.target,target.current);}
  settle(id:string,resume=false):void {
    const target=this.ports.review(id),intent=this.feature.settlement.prepare(target.current,resume);
    this.ports.confirm(resume?'补全同一结算登记':'正式结算并建立证据队列','核对原复盘、正式模型运行、冻结快照、赛事 Profile 与时点。结算账本不可覆盖，能力候选不会自动应用。',[
      ['比赛',target.target.matchId],['复盘 / 版本',target.target.reviewId+' / '+target.target.reviewVersion],['正式运行',target.target.sourceRunId??'未记录'],
    ],resume?'确认补全原登记':'确认正式结算',async()=>{
      if(!await this.feature.settlement.submit(intent))return;const current=this.feature.capture();await this.ports.refreshReview();if(!current())return;
      this.ports.showResult('正式结算已核实',this.feature.settlement.state.record);this.ports.notify('正式结算已核实；证据评分与能力候选需分别处理');
    });
  }
  async recoverSettlement():Promise<void>{const target=this.ports.review();if(await this.feature.settlement.recover(target.current))await this.ports.refreshReview();}
  monitor():void {
    const ticket=this.feature.monitoring.prepare();
    this.ports.confirm('刷新 H 正式监控','按后端当前正式分区生成评分和漂移快照；全局样本不代替当前赛事、Profile、模型、参数与时点的门禁。',[
      ['赛事 / 时点',ticket.request.competition_id+' / '+ticket.request.horizon],['请求窗口',ticket.request.baseline_size+' / '+ticket.request.current_size],['参数晋升','不会自动执行'],
    ],'确认刷新监控',async()=>{if(await this.feature.monitoring.submit(ticket))this.ports.notify('监控已返回；请按完整分区与实际样本核对报告');});
  }
  evidenceChoice(id:string):void {
    const item=this.feature.evidence.item(id);if(item.status!=='pending'||this.feature.evidence.pending(id))throw Error('该证据已判定或原请求待核实');
    this.ports.showHtml('选择 '+item.id+' 人工判定','结算 '+item.settlement_id,`<p>正确、部分正确、错误、无法验证分别记录；选择后核对依据并最终确认。</p><div class="postmatch-actions">${(Object.keys(labels) as EvidenceVerdict[]).map(verdict=>button(labels[verdict],'postmatch-evidence-verdict',{'data-evidence-item-id':id,'data-verdict':verdict})).join('')}</div>`,button('关闭','close-workspace-detail'));
  }
  evidenceVerdict(id:string,verdict:EvidenceVerdict):void {
    if(!(verdict in labels))throw Error('判定结论无效');
    const item=this.feature.evidence.item(id),token=crypto.randomUUID(),current=this.feature.capture(),draft=this.feature.evidence.draft(id);
    this.feature.evidence.saveDraft(id,draft.note,verdict);
    const guard=()=>current()&&this.evidenceDialog?.token===token&&!!document.querySelector(`[data-postmatch-evidence-dialog="${token}"]`);
    this.evidenceDialog={id,verdict,token,current:guard};
    this.ports.showHtml('判定 '+id+' 为'+labels[verdict],'最终操作确认',`<div data-postmatch-evidence-dialog="${token}" data-workspace-persist="false"><p>结算 ${html(item.settlement_id)} · ${html(item.field_key)} · ${html(item.provider_name??'未绑定供应商')}。仅保存当前证据的人工结论，不重写来源。</p><label class="postmatch-field"><span>判定说明（必填）</span>${textInputView({label:'判定说明',value:draft.note,placeholder:'写明原始证据和真实赛果的核对依据',attributes:{id:'evidence-decision-note','data-postmatch-note':token}})}</label></div>`,button('取消','close-workspace-detail')+button('确认操作','submit-evidence-decision',{'data-evidence-item-id':id,'data-postmatch-confirm':token},true));
  }
  captureNote(value:string,token:string):void {const active=this.evidenceDialog;if(active?.token===token&&active.current())this.feature.evidence.saveDraft(active.id,value,active.verdict);}
  sync():void {const active=this.evidenceDialog;if(active?.current()){const note=document.querySelector<HTMLInputElement>(`[data-postmatch-note="${active.token}"]`);if(note&&note.value!==this.feature.evidence.draft(active.id).note)note.value=this.feature.evidence.draft(active.id).note;}document.querySelectorAll<HTMLButtonElement>('[data-postmatch-confirm]').forEach(button=>{button.disabled=this.feature.evidence.state.busy||!active?.current()||!!this.feature.evidence.pending(active.id);});}
  async submitEvidence(id:string,token:string):Promise<void>{
    const active=this.evidenceDialog;if(!active||active.id!==id||active.token!==token||!active.current())throw Error('证据确认上下文已变化');
    const draft=this.feature.evidence.draft(id),intent=this.feature.evidence.prepare(id,active.verdict,draft.note,active.current);
    const row=await this.feature.evidence.submit(intent);if(!row||!active.current())return;
    this.ports.close();this.ports.showResult('证据判定已保存',row);this.ports.notify(this.feature.error?'判定已保存，列表刷新未完成':'人工判定已写入不可变账本');
  }
  async recoverEvidence(id:string):Promise<void>{const row=await this.feature.evidence.recover(id);if(row)this.ports.showResult('已查回原证据判定',row);}
  showEvidence(id:string):void{this.ports.showResult('结算证据与判定审计',this.feature.evidence.item(id));}
}

import type { StoredMatchPredictionCommand, LineupSnapshotType, PredictionModelFamily } from "../../types";

export class PredictionContext {
  private revisionValue = 0;
  private value: StoredMatchPredictionCommand = {match_id:"",snapshot_type:"T-N",model_family:"p4",explicit_rule_package_id:null};
  get revision(): number { return this.revisionValue; }
  get command(): StoredMatchPredictionCommand { return {...this.value}; }
  get matchId(): string | null { return this.value.match_id || null; }
  set matchId(value:string|null) { this.update({match_id:value??""}); }
  get snapshot(): LineupSnapshotType { return this.value.snapshot_type as LineupSnapshotType; }
  set snapshot(value:LineupSnapshotType) { this.update({snapshot_type:value}); }
  get model(): PredictionModelFamily { return this.value.model_family; }
  set model(value:PredictionModelFamily) { this.update({model_family:value}); }
  constructor(private readonly changed:()=>void) {}
  update(patch:Partial<StoredMatchPredictionCommand>):void {
    const next={...this.value,...patch};
    if(next.model_family!==this.value.model_family && !("explicit_rule_package_id" in patch)) next.explicit_rule_package_id=null;
    if(JSON.stringify(next)===JSON.stringify(this.value))return;
    this.value=next;this.revisionValue++;this.changed();
  }
  capture():()=>boolean { const revision=this.revisionValue;return ()=>revision===this.revisionValue; }
  invalidate():void{this.revisionValue++;this.changed();}
  require():StoredMatchPredictionCommand {
    if(!this.value.match_id)throw Error("请先选择比赛");
    if(!["T-N","T-24h","T-6h","T-1h"].includes(this.value.snapshot_type))throw Error("请选择有效数据窗口");
    return this.command;
  }
}

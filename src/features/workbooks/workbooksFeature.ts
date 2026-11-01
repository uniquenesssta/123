import type { SpreadsheetBatch, ImportTicket } from "../../app/imports/spreadsheetBatch";
export type WorkbookCategory = "team" | "player" | "match";
export interface WorkbookTarget {
  label: string; batch: SpreadsheetBatch; preview(): Promise<void>;
  commit(ticket: ImportTicket): Promise<void>; exportFile(template: boolean): Promise<boolean>;
  exportContext?(): { facts: Array<[string,string]>; current(): boolean };
}
interface WorkbooksPorts {
  targets: Record<WorkbookCategory, WorkbookTarget>;
  page(): string; section(): string | null; render(): void;
  confirm(title:string,description:string,facts:Array<[string,string]>,label:string,action:()=>Promise<void>):void;
}
/** Category presentation only; domain instances own all staging and writes. */
export class WorkbooksFeature {
  private generation = 0;
  private exporting: WorkbookCategory | null = null;
  private exportResult: {category:WorkbookCategory;message:string}|null=null;
  constructor(private readonly ports: WorkbooksPorts) {}
  get category(): WorkbookCategory { const value=this.ports.section(); return value==="player"||value==="match"?value:"team"; }
  get target(): WorkbookTarget { return this.ports.targets[this.category]; }
  get busy(): boolean { return this.exporting !== null || Object.values(this.ports.targets).some(t=>t.batch.busy); }
  get state() { return { targets:this.ports.targets, category:this.category, busy:this.busy, exporting:this.exporting,exportResult:this.exportResult }; }
  changedCategory(): void { this.generation++; }
  capture(): () => boolean {
    const page=this.ports.page(),category=this.category,generation=this.generation;
    return ()=>page===this.ports.page() && category===this.category && generation===this.generation;
  }
  requestCommit(): void {
    const target=this.target,ticket=target.batch.confirmation(),current=this.capture(),p=ticket.preview;
    this.ports.confirm("确认导入 · "+target.label,"核对类别、文件、导入模式与批次；确认后写入数据库。",
      [["类别",target.label],["文件",p.source_file_name],["批次",p.batch_id],["SHA256",p.source_sha256],["模式",p.import_mode==="add_only"?"仅新增":"新增并更新"]],
      "确认写入数据库",async()=>{
        if(!current())throw new Error("工作包类别或页面已变化，旧确认已失效");
        await target.commit(ticket);
      });
  }
  async readback(): Promise<void> { await this.target.batch.readback();this.ports.render(); }
  requestExport(template:boolean): void {
    if(this.busy || this.target.batch.phase==="unknown")throw new Error("原批次正在处理或待核查，请先完成核查");
    const target=this.target,category=this.category,current=this.capture(),scope=template?null:target.exportContext?.();
    this.ports.confirm("导出 · "+target.label,template?"导出空白模板，不包含现有数据。":"导出当前数据，不会获得导入提交资格。",
      [["类别",target.label],["内容",template?"空白模板":"现有数据"],...(scope?.facts??[])],"选择保存位置",async()=>{
        if(!current() || (scope && !scope.current()))throw new Error("工作包类别或导出目标已变化，请重新选择导出");
        const pending=target.exportFile(template);
        this.exporting=category;this.exportResult=null;this.ports.render();
        try{const written=await pending;if(written)this.exportResult={category,message:template?"模板已导出":"现有数据已导出"};}
        catch(error){this.exportResult={category,message:"导出失败，请核对保存位置后重试"};throw error;}
        finally{this.exporting=null;this.ports.render();}
      });
  }
}

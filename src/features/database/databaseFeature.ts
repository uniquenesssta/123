import type { ModalController } from "../../app/modal";
import { readDatabaseConnectionDraft } from "./connectionDraft";
import { DatabaseWorkflow, type DatabaseWorkflowPorts } from "./databaseWorkflow";

export interface DatabaseFeaturePorts extends DatabaseWorkflowPorts {
  busy<T>(operation: () => Promise<T>): Promise<T>;
  render(): void;
  notify(message: string): void;
  confirm: ModalController["confirm"];
}

/** Adapts current forms/confirmation UI to the connection workflow; no independent bootstrap state. */
export class DatabaseFeature {
  private readonly workflow: DatabaseWorkflow;
  constructor(private readonly ports: DatabaseFeaturePorts) {
    this.workflow = new DatabaseWorkflow({
      ...ports,
      configure: options => ports.busy(() => ports.configure(options)),
      disconnect: () => ports.busy(() => ports.disconnect()),
    });
  }

  async connect(root: ParentNode): Promise<void> {
    const draft = readDatabaseConnectionDraft(root);
    const secretInput = root.querySelector<HTMLInputElement>("#database-url");
    try {
      if (await this.workflow.connect(draft)) {
        this.ports.render();
        this.ports.notify("数据服务已连接，本页功能已恢复");
      }
    } finally {
      if (secretInput?.value.trim() === draft.connection_url) secretInput.value = "";
      draft.connection_url = "";
    }
  }

  async disconnect(): Promise<void> {
    if (await this.workflow.disconnect()) {
      this.ports.render();
      this.ports.notify("数据库连接配置已清除");
    }
  }

  requestReset(root: ParentNode): void {
    const ticket = this.workflow.prepareReset();
    const databaseName = ticket.databaseName;
    this.ports.confirm("彻底清空数据库", "此操作会删除当前数据库中的全部应用数据，并从迁移文件重新建立空白结构。球队、球员、比赛、阵容、推演、P4 快照、导入记录、AI 会话、任务、审核和审计数据都无法恢复。", [["当前数据库", databaseName], ["清空后保留", "数据库本身、结构和本机连接配置"]], "确认永久清空", async () => {
      const input = root.querySelector<HTMLInputElement>("#database-reset-confirmation");
      const confirmation = input?.value.trim() ?? "";
      await this.ports.busy(() => this.workflow.reset(ticket, confirmation));
    }, { tone: "danger", typed: { id: "database-reset-confirmation", confirmButtonId: "database-reset-confirm-button", expected: databaseName, label: `输入数据库名称 ${databaseName} 以继续`, hint: "名称必须完全一致。清空期间客户端会暂时断开，完成后自动重新连接并重新启动界面。", attributes: { "data-database-name": databaseName } } });
  }

  stop(): void { this.workflow.stop(); }
}

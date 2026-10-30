import type { BootstrapResponse, DatabaseHealth, DatabaseOptions } from "../../types";

export interface DatabaseWorkflowPorts {
  snapshot(): BootstrapResponse | null;
  configure(options: DatabaseOptions): Promise<DatabaseHealth>;
  disconnect(): Promise<void>;
  reset(confirmation: string): Promise<DatabaseHealth>;
  refresh(): Promise<boolean>;
  completeReset(): Promise<void>;
  invalidateConfirmation(): void;
}

export interface DatabaseResetTicket {
  readonly databaseName: string;
}

/** Serializes mutations of the one backend connection; Session remains the sole summary owner. */
export class DatabaseWorkflow {
  private tail: Promise<unknown> = Promise.resolve();
  private revision = 0;
  private pending = 0;
  private stopped = false;
  private resetting = false;
  private resetNeedsReview = false;
  private connectionUnverified = false;
  private latestConnect: DatabaseOptions | null = null;
  private confirmation: { ticket: DatabaseResetTicket; snapshot: BootstrapResponse; revision: number } | null = null;

  constructor(private readonly ports: DatabaseWorkflowPorts) {}

  connect(input: DatabaseOptions): Promise<boolean> {
    this.assertMutable();
    if (!input.connection_url.trim()) throw new Error("请输入数据库连接地址");
    const options = { ...input, connection_url: input.connection_url.trim() };
    if (this.latestConnect?.connection_url === options.connection_url &&
      this.latestConnect.max_connections === options.max_connections &&
      this.latestConnect.connect_timeout_seconds === options.connect_timeout_seconds) {
      return Promise.resolve(false);
    }
    this.latestConnect = options;
    return this.change(async () => { await this.ports.configure(options); }, true)
      .finally(() => { options.connection_url = ""; if (this.latestConnect === options) this.latestConnect = null; });
  }

  disconnect(): Promise<boolean> {
    this.assertMutable();
    this.latestConnect = null;
    return this.change(() => this.ports.disconnect(), false);
  }

  prepareReset(): DatabaseResetTicket {
    this.assertMutable();
    if (this.pending) throw new Error("连接变更尚未完成，请等待后重新确认");
    if (this.resetNeedsReview) throw new Error("上次清空结果尚需核实，请检查数据库实际状态并重新连接后再确认");
    if (this.connectionUnverified) throw new Error("当前连接状态未确认，请重新连接并读取最新状态后再清空");
    const snapshot = this.ports.snapshot();
    const databaseName = snapshot?.data.database_health?.database_name;
    if (!snapshot || !databaseName) throw new Error("当前未连接数据库，无法执行彻底清空");
    const ticket = Object.freeze({ databaseName });
    this.confirmation = { ticket, snapshot, revision: this.revision };
    return ticket;
  }

  async reset(ticket: DatabaseResetTicket, confirmation: string): Promise<boolean> {
    this.assertMutable();
    const prepared = this.confirmation;
    if (!prepared || prepared.ticket !== ticket || prepared.revision !== this.revision ||
      prepared.snapshot !== this.ports.snapshot() || this.pending) {
      throw new Error("数据库连接或状态已变化，请重新确认清空对象");
    }
    const expected = ticket.databaseName;
    if (confirmation !== expected) throw new Error("数据库名称不匹配，已拒绝清空");
    this.confirmation = null;
    this.latestConnect = null;
    this.resetting = true;
    try {
      await this.ports.reset(confirmation);
      if (this.stopped) return false;
      await this.ports.completeReset();
      return true;
    } catch (cause) {
      this.resetNeedsReview = true;
      throw new Error("清空未确认完成，请核查数据库实际状态；重新连接后才能再次确认", { cause });
    } finally { this.resetting = false; }
  }

  stop(): void {
    this.stopped = true;
    this.revision++;
    this.confirmation = null;
    this.latestConnect = null;
  }

  private assertMutable(): void {
    if (this.stopped) throw new Error("数据库工作流已停止");
    if (this.resetting) throw new Error("正在重建数据库，暂不能切换连接或重复提交");
  }

  private change(mutate: () => Promise<void>, reconnect: boolean): Promise<boolean> {
    const revision = ++this.revision;
    this.connectionUnverified = true;
    if (this.confirmation) this.ports.invalidateConfirmation();
    this.confirmation = null;
    this.pending++;
    const current = () => !this.stopped && revision === this.revision;
    const task = this.tail.then(async () => {
      if (!current()) return false;
      try {
        await mutate();
        if (!current()) return false;
        const applied = await this.ports.refresh();
        if (!current() || !applied) return false;
        this.connectionUnverified = false;
        if (reconnect) this.resetNeedsReview = false;
        return true;
      } catch (error) {
        if (current()) throw error;
        return false;
      }
    }).finally(() => { this.pending--; });
    this.tail = task.catch(() => {});
    return task;
  }
}

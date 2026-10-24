import { isCommandName, type CommandName } from "./commandName";
import { CancellationRegistry } from "./cancellationRegistry";
import { LoggedInvokeError, invokeErrorMessage, isInvokeCancelled } from "./invokeError";
import { deliverInvokeResponse } from "./invokeResponse";
import { InvokeTrace, createTraceId } from "./invokeTrace";

export type NativeInvoke = (command: CommandName, args?: Record<string, unknown>) => Promise<unknown>;
export interface InvokeRequestOptions { readonly signal?: AbortSignal }

/** The only IPC execution coordinator. Domain APIs supply their existing command, arguments and DTO type. */
export class InvokeTransport {
  private readonly requests = new CancellationRegistry();
  readonly trace: InvokeTrace;
  constructor(private readonly native: NativeInvoke) {
    this.trace = new InvokeTrace({ runtime: draft => native("record_runtime_operation", { draft }), issue: draft => native("record_issue", { draft }) });
  }
  get pendingCount(): number { return this.requests.size; }
  async invoke<T>(command: CommandName, args?: Record<string, unknown>, options: InvokeRequestOptions = {}): Promise<T> {
    if (!isCommandName(command)) throw new Error("未知客户端命令");
    const request = this.requests.begin(createTraceId("ipc"), options.signal);
    const trace = this.trace.request(command, request.id);
    const execute = async (): Promise<T> => {
      await trace.started(args);
      request.assertCurrent();
      let result: unknown;
      try { result = await this.native(command, args); }
      catch (error) {
        request.assertCurrent();
        const message = invokeErrorMessage(error);
        await trace.failed(error);
        request.assertCurrent();
        await this.trace.issue(command, message);
        request.assertCurrent();
        throw new LoggedInvokeError(message, error);
      }
      request.assertCurrent();
      await trace.completed(result);
      return deliverInvokeResponse<T>(request, result);
    };
    try { return await request.wait(execute()); }
    catch (error) {
      if (isInvokeCancelled(error)) void trace.failed(error, true);
      throw error;
    } finally { request.finish(); }
  }
  stop(): void { this.requests.stop(); this.trace.stopUi(); }
}

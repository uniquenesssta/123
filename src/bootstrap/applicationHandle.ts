import { DisposableScope, MountHandle } from "../app/lifecycle";

export interface ApplicationModule {
  readonly name: string;
  start(): Promise<void> | void;
  destroy(): Promise<void> | void;
  stop?(): void;
}

export class ApplicationHandle {
  readonly root: HTMLDivElement;
  private readonly modules: ApplicationModule[] = [];
  private readonly moduleNames = new Set<string>();
  private readonly startedModules: ApplicationModule[] = [];
  private readonly resources = new DisposableScope("应用模块销毁失败");
  private readonly lifecycle: MountHandle;
  private registrationClosed = false;
  private startingModule = "";

  constructor(root: HTMLDivElement) {
    this.root = root;
    this.lifecycle = new MountHandle({
      start: () => this.startModules(),
      stop: () => {
        const errors: unknown[] = [];
        for (const module of [...this.startedModules].reverse()) {
          try { module.stop?.(); } catch (error: unknown) { errors.push(error); }
        }
        if (errors.length) throw new AggregateError(errors, "应用模块停止失败");
      },
      destroy: async () => {
        try { await this.resources.dispose(); } finally {
          this.startedModules.length = 0;
          this.modules.length = 0;
          this.moduleNames.clear();
        }
      },
      closedMessage: "应用生命周期已结束",
      destroyFailureMessage: "应用模块销毁失败",
      startFailureMessage: () => `应用模块启动失败：${this.startingModule}`,
    });
  }

  register(module: ApplicationModule): void {
    if (this.registrationClosed) {
      throw new Error("应用启动后不能继续注册模块");
    }
    if (this.moduleNames.has(module.name)) {
      throw new Error(`应用模块重复注册：${module.name}`);
    }
    this.moduleNames.add(module.name);
    this.modules.push(module);
  }

  start(): Promise<void> {
    this.registrationClosed = true;
    return this.lifecycle.start();
  }

  destroy(): Promise<void> {
    this.registrationClosed = true;
    return this.lifecycle.destroy();
  }

  private async startModules(): Promise<void> {
    for (const module of this.modules) {
      if (this.lifecycle.signal.aborted) return;
      this.startingModule = module.name;
      this.startedModules.push(module);
      this.resources.defer(() => module.destroy());
      await module.start();
    }
  }
}

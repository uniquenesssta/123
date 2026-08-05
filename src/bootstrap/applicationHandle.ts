export interface ApplicationModule {
  readonly name: string;
  start(): Promise<void> | void;
  destroy(): Promise<void> | void;
}

export class ApplicationHandle {
  readonly root: HTMLDivElement;
  private readonly modules: ApplicationModule[] = [];
  private readonly startedModules: ApplicationModule[] = [];
  private readonly moduleNames = new Set<string>();
  private startTask: Promise<void> | null = null;
  private destroyTask: Promise<void> | null = null;
  private destroyed = false;

  constructor(root: HTMLDivElement) {
    this.root = root;
  }

  register(module: ApplicationModule): void {
    if (this.startTask || this.destroyed) {
      throw new Error("应用启动后不能继续注册模块");
    }
    if (this.moduleNames.has(module.name)) {
      throw new Error(`应用模块重复注册：${module.name}`);
    }
    this.moduleNames.add(module.name);
    this.modules.push(module);
  }

  start(): Promise<void> {
    if (this.destroyed) return Promise.reject(new Error("应用生命周期已结束"));
    if (!this.startTask) this.startTask = this.startModules();
    return this.startTask;
  }

  destroy(): Promise<void> {
    if (!this.destroyTask) this.destroyTask = this.destroyModules();
    return this.destroyTask;
  }

  private async startModules(): Promise<void> {
    for (const module of this.modules) {
      try {
        await module.start();
        this.startedModules.push(module);
      } catch (error: unknown) {
        const cleanupErrors = await this.destroyModuleSequence([module, ...[...this.startedModules].reverse()]);
        this.startedModules.length = 0;
        this.destroyed = true;
        if (cleanupErrors.length > 0) {
          throw new AggregateError([error, ...cleanupErrors], `应用模块启动失败：${module.name}`);
        }
        throw error;
      }
    }
  }

  private async destroyModules(): Promise<void> {
    if (this.destroyed) return;
    this.destroyed = true;
    if (this.startTask) {
      try {
        await this.startTask;
      } catch {
        return;
      }
    }
    const cleanupErrors = await this.destroyModuleSequence([...this.startedModules].reverse());
    this.startedModules.length = 0;
    if (cleanupErrors.length > 0) {
      throw new AggregateError(cleanupErrors, "应用模块销毁失败");
    }
  }

  private async destroyModuleSequence(
    modules: readonly ApplicationModule[],
  ): Promise<unknown[]> {
    const errors: unknown[] = [];
    for (const module of modules) {
      try {
        await module.destroy();
      } catch (error: unknown) {
        errors.push(error);
      }
    }
    return errors;
  }
}

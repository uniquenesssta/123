import type { ApplicationHandle, ApplicationModule } from "./applicationHandle";

interface BrowserApplicationRuntime {
  start(): Promise<void>;
  destroy(): Promise<void>;
  stop(): void;
}

export function registerApplicationModules(handle: ApplicationHandle): void {
  let runtime: BrowserApplicationRuntime | null = null;
  let stopped = false;

  const browserApplication: ApplicationModule = {
    name: "browser-application",
    async start() {
      const module = await import("../main");
      runtime = module.createBrowserApplicationModule(handle.root);
      if (stopped) { runtime.stop(); return; }
      await runtime.start();
    },
    stop() {
      stopped = true;
      runtime?.stop();
    },
    async destroy() {
      await runtime?.destroy();
      runtime = null;
    },
  };

  handle.register(browserApplication);
}

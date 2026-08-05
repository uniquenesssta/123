import type { ApplicationHandle, ApplicationModule } from "./applicationHandle";

interface BrowserApplicationRuntime {
  start(): Promise<void>;
  destroy(): Promise<void>;
}

export function registerApplicationModules(handle: ApplicationHandle): void {
  let runtime: BrowserApplicationRuntime | null = null;

  const browserApplication: ApplicationModule = {
    name: "browser-application",
    async start() {
      const module = await import("../main");
      runtime = module.createBrowserApplicationModule(handle.root);
      await runtime.start();
    },
    async destroy() {
      await runtime?.destroy();
      runtime = null;
    },
  };

  handle.register(browserApplication);
}

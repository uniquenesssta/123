import { WarningCenterController } from "../index";
import { renderShell } from "../../shell";

export function installWarningFixture(theme: "light" | "dark") {
  const root = document.querySelector<HTMLElement>("#warning-test-root")!;
  const navigation: string[] = [];
  let now = Date.now();
  const controller = new WarningCenterController(root, { navigate: target => navigation.push(target) }, () => now);
  const expect = (ok: unknown, label: string) => { if (!ok) throw new Error(label); checks.push(label); };
  const checks: string[] = [];
  const button = (action: string) => document.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)!;
  const paint = () => {
    root.innerHTML = renderShell({ state: { data: { database_configured: true, app_version: "0.23.0" } }, theme, page: "dashboard", sidebarCollapsed: false, navigationPending: false,
      content: '<section style="min-height:1600px"><input id="warning-draft" value="尚未保存的输入"><button id="business-gate" disabled>受业务门禁保护</button><p>独立页面滚动区域</p></section>' });
    controller.restore();
  };
  paint(); controller.start(); controller.start();
  expect(document.querySelectorAll(".warning-center-root").length === 1, "start is idempotent");
  button("open-warning-center").focus(); button("open-warning-center").click();
  expect(!!document.querySelector("dialog[open] .warning-empty"), "empty dialog and native modality");
  expect(document.activeElement === button("close-warning-center").closest("dialog")?.querySelector("[autofocus]"), "initial focus is on footer Close");
  button("close-warning-center").click();
  expect(document.activeElement === button("open-warning-center"), "close restores trigger focus");
  controller.observeConnection('postgres://user:private@localhost/db password=secret');
  controller.observeConnection('postgres://user:private@localhost/db password=secret');
  expect(controller.records.length === 1 && controller.records[0].occurrences === 1 && controller.unreadCount === 1, "source observations do not duplicate warnings");
  expect(!controller.records[0].message.includes("private") && !controller.records[0].message.includes("secret"), "visible connection summary is redacted");
  controller.observeConnection(null);
  expect(controller.unreadCount === 0 && controller.records[0].resolved, "actual connection recovery resolves the observed source");
  controller.record({ key: "yesterday", source: "当前工作区", severity: "warning", title: "当前草稿将被替换", message: "请核对当前内容后再执行操作。", occurredAt: new Date(now - 86_400_000).toISOString() });
  controller.record({ key: "today", source: "数据服务", severity: "error", title: "数据服务连接失败", message: "请检查连接信息后重试，关闭提示不会恢复连接。" });
  const draft = document.querySelector<HTMLInputElement>("#warning-draft")!;
  const page = document.querySelector<HTMLElement>(".page-container")!;
  draft.value = "保留真实工作区输入"; draft.focus({ preventScroll: true }); page.scrollTop = 217;
  expect(page.scrollTop === 217, "full production CSS preserves the deliberately tall scroll fixture");
  controller.notify("<script>window.injected=true</script>", "danger", "验收");
  expect(!document.querySelector(".ui-toast script") && !(window as Window & { injected?: boolean }).injected, "warning/notice HTML is escaped");
  expect(document.activeElement === draft, "notice never steals focus");
  controller.dismissNotice();
  button("open-warning-center").focus(); button("open-warning-center").click();
  const headings = Array.from(document.querySelectorAll(".warning-date-group h3"), node => node.textContent);
  expect(headings.includes("今天") && headings.includes("昨天"), "dialog groups local calendar days");
  expect(document.querySelector("dialog")!.getBoundingClientRect().width === 480, "approved 480px Drawer geometry");
  expect(getComputedStyle(document.querySelector("dialog")!).backgroundColor === (theme === "dark" ? "rgb(31, 33, 44)" : "rgb(255, 255, 255)"), "resolved approved Light/Dark surface");
  button("read-all-warnings").click();
  expect(controller.unreadCount === 0 && button("read-all-warnings").disabled, "mark read updates badge and disables duplicate action");
  button("close-warning-center").click();
  expect(draft.value === "保留真实工作区输入", "dialog/read/dismiss preserve draft");
  expect(page.scrollTop === 217, `dialog/read/dismiss preserve page scroll: ${page.scrollTop}`);
  expect(document.querySelector<HTMLButtonElement>("#business-gate")!.disabled, "dialog/read/dismiss preserve business gate");
  let finishTask!: () => void;
  const pending = controller.runTask(() => new Promise<void>(resolve => { finishTask = resolve; }));
  expect(controller.taskActive && !document.querySelector<HTMLElement>(".ui-task-activity")!.hidden && document.querySelector(".ui-task-activity")!.getAttribute("aria-hidden") === "false", "task visibility and ARIA are synchronized");
  controller.notify("任务中的持续提示", "danger", "验收");
  const visibleNotice = document.querySelector(".ui-toast");
  controller.notify("任务中的持续提示", "danger", "验收");
  expect(document.querySelector(".ui-toast") === visibleNotice, "duplicate notice is merged without replacing or reannouncing DOM");
  button("dismiss-notice").click();
  expect(controller.taskActive, "notice dismissal does not cancel pending operation");
  const secondTask = controller.runTask(async () => {}).then(() => false, error => error.message === "已有任务正在执行");
  return { controller, root, checks, navigation, button, expect, paint, pending, secondTask, finishTask,
    addLongList() { for (let index = 0; index < 24; index++) controller.record({ key: `long:${index}`, source: "长内容验证", severity: "warning", title: `第 ${index + 1} 项提醒`, message: "需核对的长内容与当前对象信息。".repeat(20), occurredAt: new Date(now - (index % 5) * 86_400_000).toISOString() }); },
    finalSpecimen() { controller.dismissNotice(); controller.open(); },
  };
}

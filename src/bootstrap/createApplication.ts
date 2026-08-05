import { ApplicationHandle } from "./applicationHandle";
import { registerApplicationModules } from "./registerApplicationModules";

export function createApplication(documentRoot: Document = document): ApplicationHandle {
  const root = documentRoot.querySelector<HTMLDivElement>("#app");
  if (!root) throw new Error("缺少 #app 根节点");
  const handle = new ApplicationHandle(root);
  registerApplicationModules(handle);
  return handle;
}

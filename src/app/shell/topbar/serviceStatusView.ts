import { spinnerView } from "../../../ui/primitives/spinner";

export function serviceStatusView(configured: boolean, loading: boolean): string {
  if (loading) return `<span class="shell-service-status loading" role="status">${spinnerView(16)}<span>正在载入</span></span>`;
  return `<span class="shell-service-status ${configured ? "configured" : "unconfigured"}"><span class="shell-service-badge">${configured ? "数据库已配置" : "等待数据库"}</span></span>`;
}

import type { DatabaseOptions } from "../../types";

/** A submission snapshot only. Workspace keeps non-sensitive controls; URLs are never persisted here. */
export function readDatabaseConnectionDraft(root: ParentNode): DatabaseOptions {
  const value = (id: string): string => {
    const input = root.querySelector<HTMLInputElement>(`#${id}`);
    if (!input) throw new Error(`缺少输入控件：${id}`);
    return input.value;
  };
  return {
    connection_url: value("database-url").trim(),
    max_connections: Number(value("max-connections")),
    connect_timeout_seconds: Number(value("connect-timeout")),
  };
}

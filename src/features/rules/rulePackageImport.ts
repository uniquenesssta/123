import type { RulePackageDraft } from "../../types";

function record(value: unknown): value is Record<string, unknown> { return !!value && typeof value === "object" && !Array.isArray(value); }
function strings(value: Record<string, unknown>, keys: string[]): boolean { return keys.every(key => typeof value[key] === "string" && (value[key] as string).trim().length > 0); }

/** Shape precheck protects rendering only; the backend validates schema/version/model and registration. */
export function parseRulePackage(text: string): RulePackageDraft {
  const value: unknown = JSON.parse(text);
  if (!record(value) || !strings(value, ["package_key", "version", "display_name"]) || value.format_version !== undefined && !strings(value, ["format_version"])) throw new Error("规则包缺少名称或版本");
  const profile = value.competition_profile, routing = value.routing;
  if (!record(profile) || !strings(profile, ["profile_id", "name", "competition_kind"])) throw new Error("规则包赛事 Profile 格式无效");
  if (!record(routing) || !strings(routing, ["model_id", "model_version", "parameter_version"])) throw new Error("规则包模型路由格式无效");
  // Optional serde defaults and JSON Value fields pass through unchanged. Do not invent client defaults
  // or reject backend-supported omissions; registration owns full schema and parameter validation.
  return value as unknown as RulePackageDraft;
}

/** Owns the current file identity and candidate, never persisted in Workspace or Bootstrap. */
export class RulePackageImport {
  private revision = 0;
  private candidate: RulePackageDraft | null = null;
  private loading = false;
  private closed = false;
  get pending(): RulePackageDraft | null { return this.candidate ? structuredClone(this.candidate) : null; }
  get reading(): boolean { return this.loading; }
  clear(): void { this.revision++; this.candidate = null; this.loading = false; }
  async read(file: { text(): Promise<string> }): Promise<boolean> {
    if (this.closed) return false;
    const ticket = ++this.revision;
    this.loading = true;
    try {
      const text = await file.text();
      if (this.closed || ticket !== this.revision) return false;
      this.candidate = parseRulePackage(text);
      return true;
    } catch (error) {
      if (this.closed || ticket !== this.revision) return false;
      throw error;
    } finally { if (ticket === this.revision) this.loading = false; }
  }
  registration(): { draft: RulePackageDraft; revision: number } {
    if (this.closed || this.loading || !this.candidate) throw new Error("请等待规则包读取完成并核对文件");
    return { draft: structuredClone(this.candidate), revision: this.revision };
  }
  registered(revision: number): void { if (this.revision === revision) this.clear(); }
  stop(): void { this.closed = true; this.clear(); }
}

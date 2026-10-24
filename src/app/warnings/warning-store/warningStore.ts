import { normalizedWarning, safeWarningText, type WarningInput, type WarningRecord, type FeedbackTone, type FeedbackNotice, type FeedbackAction } from "../warning-dto";

/** Sole session owner of global warning history, unread state, visible notice and task activity. */
export class WarningStore {
  private rows: WarningRecord[] = [];
  private currentNotice: FeedbackNotice | null = null;
  private sequence = 0;
  private running = false;

  constructor(private readonly clock: () => number = Date.now) {}
  get records(): readonly WarningRecord[] { return Object.freeze([...this.rows]); }
  get unreadCount(): number { return this.rows.filter(row => !row.read && !row.resolved).length; }
  get notice(): FeedbackNotice | null { return this.currentNotice; }
  get taskActive(): boolean { return this.running; }

  record(input: WarningInput): WarningRecord {
    const value = normalizedWarning(input, this.clock());
    const previous = this.rows.find(row => row.key === value.key);
    const row: WarningRecord = Object.freeze({
      key: value.key, source: value.source, severity: value.severity, title: value.title, message: value.message, target: value.target,
      firstSeenAt: previous?.firstSeenAt ?? value.occurredAt, lastSeenAt: value.occurredAt,
      occurrences: (previous?.occurrences ?? 0) + 1, read: false, resolved: false,
    });
    this.rows = [row, ...this.rows.filter(item => item.key !== row.key)].slice(0, 200);
    return row;
  }

  /** Repeated rendering of the same source is an observation, not a new warning event. */
  observe(input: WarningInput): void {
    const value = normalizedWarning(input, this.clock());
    const previous = this.rows.find(row => row.key === value.key);
    if (previous && !previous.resolved && previous.message === value.message && previous.title === value.title && previous.severity === value.severity) return;
    this.record(input);
  }

  resolve(key: string): void {
    this.rows = this.rows.map(row => row.key === key ? Object.freeze({ ...row, resolved: true, read: true }) : row);
  }
  markAllRead(): void { this.rows = this.rows.map(row => row.read ? row : Object.freeze({ ...row, read: true })); }
  markRead(key: string): void { this.rows = this.rows.map(row => row.key === key ? Object.freeze({ ...row, read: true }) : row); }

  notify(message: string, tone: FeedbackTone, action?: FeedbackAction): FeedbackNotice {
    const safe = safeWarningText(message);
    if (this.currentNotice?.message === safe && this.currentNotice.tone === tone && this.currentNotice.action?.target === action?.target && this.currentNotice.action?.label === action?.label) return this.currentNotice;
    this.currentNotice = Object.freeze({ id: ++this.sequence, message: safe, tone,
      action: action ? Object.freeze({ ...action, label: safeWarningText(action.label) }) : undefined,
      autoDismiss: !action && (tone === "info" || tone === "success"),
    });
    return this.currentNotice;
  }
  dismissNotice(id: number): boolean {
    if (this.currentNotice?.id !== id) return false;
    this.currentNotice = null;
    return true;
  }
  beginTask(): boolean { if (this.running) return false; this.running = true; return true; }
  endTask(): void { this.running = false; }
}

export interface WarningDateGroup { readonly date: string; readonly label: string; readonly records: readonly WarningRecord[]; }

/** Local calendar dates, including midnight/DST boundaries, rather than elapsed 24-hour buckets. */
export function warningDateGroups(records: readonly WarningRecord[], now = Date.now()): readonly WarningDateGroup[] {
  const dayKey = (value: Date): string => `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}-${String(value.getDate()).padStart(2, "0")}`;
  const today = new Date(now), yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  const groups = new Map<string, WarningRecord[]>();
  for (const row of [...records].sort((a, b) => b.lastSeenAt.localeCompare(a.lastSeenAt))) {
    const date = dayKey(new Date(row.lastSeenAt));
    const values = groups.get(date) ?? [];
    values.push(row); groups.set(date, values);
  }
  return [...groups].map(([date, rows]) => ({ date, label: date === dayKey(today) ? "今天" : date === dayKey(yesterday) ? "昨天" : date, records: rows }));
}

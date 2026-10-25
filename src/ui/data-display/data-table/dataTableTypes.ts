import type { ButtonOptions } from "../../primitives/button";
import type { StatusBadgeOptions } from "../status-badge";
export type TableCell = string | number | boolean | null | { kind: "text" | "metadata" | "number"; text: string; supporting?: string } | { kind: "status"; status: StatusBadgeOptions } | { kind: "action"; button: ButtonOptions };
export interface TableColumn { key: string; label: string; type?: "Text" | "Number" | "Action"; sortable?: boolean; sort?: "none" | "ascending" | "descending"; width?: number }
export interface TableRow { id: string; cells: Readonly<Record<string, TableCell>>; selected?: boolean; disabled?: boolean }
export interface DataTableOptions { label: string; columns: readonly TableColumn[]; rows: readonly TableRow[]; busy?: boolean; selection?: boolean; emptyTitle?: string; emptyDescription?: string; loadingLabel?: string }
export interface DataTableActions { sort?(column: string, direction: "ascending" | "descending"): void; select?(id: string, checked: boolean): void; selectAll?(checked: boolean): void }

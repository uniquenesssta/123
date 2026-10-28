export interface WorkspaceControlState {
  readonly key: string;
  readonly value: string;
  readonly checked: boolean | null;
  readonly selectedValues: readonly string[] | null;
}

export interface WorkspacePanelEntry {
  readonly title: string;
  readonly subtitle: string;
  readonly body: string;
  readonly footer: string;
  readonly panelClass: string;
  formState?: WorkspaceControlState[];
  bodyScrollTop?: number;
  activeControlKey?: string | null;
}

export interface WorkspaceSectionItem {
    readonly id: string;
    readonly index: string;
    readonly label: string;
    readonly description?: string;
    readonly badge?: string;
    readonly disabled?: boolean;
}
export interface WorkspaceAnchorItem {
    readonly id: string;
    readonly label: string;
}

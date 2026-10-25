import { open, save } from "@tauri-apps/plugin-dialog";

export interface FileFilter { name: string; extensions: string[] }
export interface FileDialogOptions { title?: string; defaultPath?: string; filters?: FileFilter[] }
export interface DirectoryDialogOptions { title?: string; defaultPath?: string }

/** Native selection only; importing/exporting content remains with the existing commands. */
export async function openFile(options: FileDialogOptions = {}): Promise<string | null> {
  const selected = await open({ ...options, multiple: false, directory: false });
  return typeof selected === "string" ? selected : null;
}

export async function saveFile(options: FileDialogOptions = {}): Promise<string | null> {
  const selected = await save(options);
  return typeof selected === "string" ? selected : null;
}

export async function selectDirectory(options: DirectoryDialogOptions = {}): Promise<string | null> {
  const selected = await open({ ...options, multiple: false, directory: true, recursive: false });
  return typeof selected === "string" ? selected : null;
}

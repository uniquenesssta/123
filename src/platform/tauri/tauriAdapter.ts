import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { InvokeTransport } from "./invokeRequest";
import type { CommandName } from "./commandName";
import type { InvokeRequestOptions } from "./invokeRequest";

const transport = new InvokeTransport((command, args) => tauriInvoke(command, args));
export function invoke<T>(command: CommandName, args?: Record<string, unknown>, options?: InvokeRequestOptions): Promise<T> { return transport.invoke<T>(command, args, options); }
export function recordUiOperation(phase: "ui_action" | "navigation", operation: string, context?: Record<string, unknown>): string { return transport.trace.ui(phase, operation, context); }
export function recordUiOperationFailure(operation: string, error: unknown, traceId: string, context?: Record<string, unknown>): void { transport.trace.uiFailure(operation, error, traceId, context); }
export function recordFrontendDiagnostic(operation: string, context?: Record<string, unknown>): void { transport.trace.diagnostic(operation, context); }
export function stopTauriTransport(): void { transport.stop(); }

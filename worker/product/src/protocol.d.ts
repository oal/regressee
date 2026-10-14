// SPDX-License-Identifier: MPL-2.0
// Mirrors crates/runtime/src/protocol.rs. Runnable adapter: index.mjs.
export const VERSION: 2;
export const MAX_FRAME: number;
export const COMMANDS: readonly string[];
// All three fields are non-nil UUIDs, never credential material.
export interface SessionReference { id: string; project: string; environment: string }
export interface TargetKey { page: string; profile: string }
export type Command =
  | { type: 'Handshake' | 'Cancel' | 'Shutdown' }
  | { type: 'SetupLogin'; payload: { session: SessionReference } }
  | { type: 'OpenLogin'; payload: {session: SessionReference; address: string; representative: string; origins: string[]; identity_origins: string[]} }
  | { type: 'ConfirmLogin'; payload: {session: SessionReference} }
  | { type: 'InstallSession'; payload: {session: SessionReference; origins: string[]; material: string} }
  | { type: 'DiscoverAuthenticated'; payload: {address: string; section: string | null; session: SessionReference} }
  | { type: 'DiscoverPage'; payload: { address: string; section: string | null } }
  | { type: 'FetchRobots'; payload: { address: string } }
  | { type: 'CaptureTarget'; payload: { key: TargetKey; address: string; profile: Readonly<Record<string, unknown>>; session: SessionReference | null } };
export interface Request { version: 2; request_id: number; job: string; command: Command }
export type FailureCode = 'UnsupportedVersion' | 'InvalidRequest' | 'NeedsLogin' | 'Timeout' | 'CaptureFailed' | 'WorkerStopped' | 'PayloadLimit' | 'Cancelled' | 'AccessDenied' | 'NotFound' | 'RedirectOutsideScope' | 'ImageLimit' | 'RateLimited' | 'ServerError';
export type WarningCode = 'ReadinessBudget' | 'Unstable' | 'ReplacementGeometryChanged' | 'ReplacementGeometryNotMeasured' | 'ReplacementSourceDependent' | 'ReplacementUnsupportedMedia' | 'FontsIncomplete' | 'ImagesIncomplete' | 'ResourceFailed' | 'EmbeddedContentBlocked';
export interface Artifact { id: string; bytes: number; kind: 'Screenshot' | 'Thumbnail' | 'Tile' | 'Difference' }
export type EventKind =
  | { type: 'Acknowledged' | 'RequiresLogin' | 'LoginOpened' | 'Completed' | 'Cancelled' }
  | { type: 'SessionReady'; payload: {material: string} } // private IPC, never logs/evidence
  | { type: 'Ready'; payload: { worker_version: string; browser_version: string } }
  | { type: 'Progress'; payload: { completed: number; total: number | null } }
  | { type: 'LinksFound'; payload: { addresses: string[]; truncated: boolean; final_url: string } }
  | { type: 'RobotsFound'; payload: { text: string; status: number } }
  | { type: 'RedirectFound'; payload: { address: string } }
  | { type: 'TargetCaptured'; payload: { key: TargetKey; output_token: string; artifact: Artifact; width: number; height: number } }
  | { type: 'Warning'; payload: { code: WarningCode } }
  | { type: 'Failure'; payload: { code: FailureCode; retryable: boolean } };
export interface Event { version: 2; request_id: number; job: string; event: EventKind }
export class ProtocolError extends Error { code: string }
export function parseRequest(frame: Uint8Array): Request;

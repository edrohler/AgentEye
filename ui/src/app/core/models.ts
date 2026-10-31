/** Mirrors `agenteye_core::AgentKind` (serialized lowercase). */
export type AgentKind = 'claude' | 'codex';

/** Mirrors `agenteye_core::SessionSummary` (serialized camelCase). */
export interface SessionSummary {
  readonly agent: AgentKind;
  readonly path: string;
  readonly sessionId: string | null;
  readonly title: string | null;
  readonly cwd: string | null;
  readonly model: string | null;
  readonly sizeBytes: number;
  /** Milliseconds since the Unix epoch. */
  readonly lastActivityMs: number;
}

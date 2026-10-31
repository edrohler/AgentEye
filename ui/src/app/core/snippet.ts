/**
 * Marks around each match in a search snippet. Control characters are used because they cannot
 * occur in real message text. Mirrors `HIGHLIGHT_START` / `HIGHLIGHT_END` in `agenteye-index`.
 */
export const HIGHLIGHT_START = '\u0002';
export const HIGHLIGHT_END = '\u0003';

export interface SnippetPart {
  readonly text: string;
  readonly highlighted: boolean;
}

/** Splits a snippet into alternating plain and highlighted parts, ready to render safely as text. */
export function splitSnippet(snippet: string): readonly SnippetPart[] {
  const parts: SnippetPart[] = [];
  let rest = snippet;
  while (rest.length > 0) {
    const start = rest.indexOf(HIGHLIGHT_START);
    if (start === -1) {
      parts.push({ text: rest, highlighted: false });
      break;
    }
    if (start > 0) {
      parts.push({ text: rest.slice(0, start), highlighted: false });
    }
    const afterStart = rest.slice(start + 1);
    const end = afterStart.indexOf(HIGHLIGHT_END);
    if (end === -1) {
      parts.push({ text: afterStart, highlighted: true });
      break;
    }
    parts.push({ text: afterStart.slice(0, end), highlighted: true });
    rest = afterStart.slice(end + 1);
  }
  return parts;
}

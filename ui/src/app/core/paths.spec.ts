import { basename } from './paths';

describe('basename', () => {
  it('handles Windows and POSIX separators', () => {
    expect(basename('C:\\Users\\me\\.claude\\s1.jsonl')).toBe('s1.jsonl');
    expect(basename('/home/me/.codex/rollout-1.jsonl')).toBe('rollout-1.jsonl');
  });

  it('falls back to the input when there are no segments', () => {
    expect(basename('')).toBe('');
    expect(basename('/')).toBe('/');
  });
});

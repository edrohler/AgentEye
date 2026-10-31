import { HIGHLIGHT_END, HIGHLIGHT_START, splitSnippet } from './snippet';

const mark = (word: string) => `${HIGHLIGHT_START}${word}${HIGHLIGHT_END}`;

describe('splitSnippet', () => {
  it('returns one plain part when nothing is highlighted', () => {
    expect(splitSnippet('no matches')).toEqual([{ text: 'no matches', highlighted: false }]);
  });

  it('alternates plain and highlighted parts', () => {
    expect(splitSnippet(`fix the ${mark('build')} now`)).toEqual([
      { text: 'fix the ', highlighted: false },
      { text: 'build', highlighted: true },
      { text: ' now', highlighted: false },
    ]);
  });

  it('keeps literal brackets as plain text', () => {
    expect(splitSnippet(`[${mark('DevIndext')}].Cli`)).toEqual([
      { text: '[', highlighted: false },
      { text: 'DevIndext', highlighted: true },
      { text: '].Cli', highlighted: false },
    ]);
  });

  it('handles adjacent highlights and an unterminated one', () => {
    expect(splitSnippet(`${mark('a')}${mark('b')}`).map((p) => p.text)).toEqual(['a', 'b']);
    expect(splitSnippet(`${HIGHLIGHT_START}open`)).toEqual([{ text: 'open', highlighted: true }]);
  });

  it('returns nothing for an empty snippet', () => {
    expect(splitSnippet('')).toEqual([]);
  });
});

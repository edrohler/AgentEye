/** The last segment of a Windows or POSIX path. */
export function basename(path: string): string {
  const segments = path.split(/[\\/]/).filter((segment) => segment.length > 0);
  return segments.length > 0 ? segments[segments.length - 1] : path;
}

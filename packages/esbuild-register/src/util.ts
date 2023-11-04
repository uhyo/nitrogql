/**
 * Computes parent directory of the given URL.
 */
export function parentDir(url: URL): URL {
  const result = new URL(url);
  result.pathname = result.pathname.replace(/\/[^/]+$/, "");
  return result;
}

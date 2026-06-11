/** Parse `1.2.3`, `v1.2.3`, or `1.2` (patch defaults to 0). */
export function parseSemver(version: string): [number, number, number] | null {
  let s = version.trim();
  if (s.startsWith("v") || s.startsWith("V")) {
    s = s.slice(1);
  }
  const core = s.split("-")[0]?.split("+")[0] ?? s;
  const parts = core.split(".");
  if (parts.length < 2 || parts.length > 3) {
    return null;
  }
  const nums: number[] = [];
  for (const p of parts) {
    if (!/^\d+$/.test(p)) {
      return null;
    }
    nums.push(Number(p));
  }
  while (nums.length < 3) {
    nums.push(0);
  }
  return [nums[0]!, nums[1]!, nums[2]!];
}

/** True when `candidate` is strictly newer than `installed` (major → minor → patch). */
export function isVersionNewer(candidate: string, installed: string): boolean {
  const a = parseSemver(candidate);
  const b = parseSemver(installed);
  if (!a || !b) {
    return false;
  }
  for (let i = 0; i < 3; i++) {
    if (a[i]! > b[i]!) {
      return true;
    }
    if (a[i]! < b[i]!) {
      return false;
    }
  }
  return false;
}

/** Stable dotted context for host logs, e.g. `SettingsPage.backup`. */
export function logContext(component: string, action: string): string {
  return `${component}.${action}`;
}

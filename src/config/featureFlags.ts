import { env } from "./env";

/** Optional feature toggles via `VITE_FEATURE_*` — extend as the app grows. */
export const featureFlags = {
  experimentalUi: env.parseBool(
    import.meta.env.VITE_FEATURE_EXPERIMENTAL_UI,
    false,
  ),
} as const;

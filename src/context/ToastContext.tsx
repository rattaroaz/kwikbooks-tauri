import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { reportError } from "../lib/reportError";

type ToastKind = "info" | "error" | "success";

export type ToastItem = { id: number; kind: ToastKind; message: string };

type ToastContextValue = {
  toasts: ToastItem[];
  push: (kind: ToastKind, message: string) => void;
  /** Log API/IPC failures to host and show an error toast. */
  pushApiError: (error: unknown, context: string) => void;
  dismiss: (id: number) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

let seq = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const push = useCallback((kind: ToastKind, message: string) => {
    const id = ++seq;
    setToasts((t) => [...t, { id, kind, message }]);
    window.setTimeout(() => {
      setToasts((t) => t.filter((x) => x.id !== id));
    }, 6000);
  }, []);

  const pushApiError = useCallback(
    (error: unknown, context: string) => {
      reportError(context, error, (message) => push("error", message));
    },
    [push],
  );

  const dismiss = useCallback((id: number) => {
    setToasts((t) => t.filter((x) => x.id !== id));
  }, []);

  const value = useMemo(
    () => ({ toasts, push, pushApiError, dismiss }),
    [toasts, push, pushApiError, dismiss],
  );

  return (
    <ToastContext.Provider value={value}>{children}</ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast outside ToastProvider");
  }
  return ctx;
}

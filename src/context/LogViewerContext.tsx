import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  expandWindowForLogPanel,
  restoreWindowAfterLogPanel,
} from "../lib/logPanelWindow";

type LogViewerContextValue = {
  open: boolean;
  openLogs: () => Promise<void>;
  closeLogs: () => Promise<void>;
};

const LogViewerContext = createContext<LogViewerContextValue | null>(null);

export function LogViewerProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);

  const openLogs = useCallback(async () => {
    await expandWindowForLogPanel();
    setOpen(true);
  }, []);

  const closeLogs = useCallback(async () => {
    setOpen(false);
    await restoreWindowAfterLogPanel();
  }, []);

  const value = useMemo(
    () => ({ open, openLogs, closeLogs }),
    [open, openLogs, closeLogs],
  );

  return (
    <LogViewerContext.Provider value={value}>
      {children}
    </LogViewerContext.Provider>
  );
}

export function useLogViewer(): LogViewerContextValue {
  const ctx = useContext(LogViewerContext);
  if (!ctx) {
    throw new Error("useLogViewer must be used within LogViewerProvider");
  }
  return ctx;
}

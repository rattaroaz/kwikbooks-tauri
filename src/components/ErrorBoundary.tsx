import { Component, type ErrorInfo, type ReactNode } from "react";
import { captureException } from "../config/telemetry";

type Props = {
  children: ReactNode;
  fallback?: ReactNode;
};

type State = { error: Error | null };

/** Catches render errors and logs them to the host log pipeline. */
export class ErrorBoundary extends Component<Props, State> {
  override state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  override componentDidCatch(error: Error, info: ErrorInfo): void {
    const where =
      info.componentStack?.split("\n").find((l) => l.trim().length > 0)?.trim() ??
      "render";
    captureException(error, `ErrorBoundary.${where}`);
  }

  override render(): ReactNode {
    if (this.state.error !== null) {
      return (
        this.props.fallback ?? (
          <div className="kb-page kb-error-screen" role="alert">
            <h1>Something went wrong</h1>
            <p>{this.state.error.message}</p>
          </div>
        )
      );
    }
    return this.props.children;
  }
}

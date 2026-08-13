import { render, type RenderOptions } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import type { ReactElement, ReactNode } from "react";
import { ToastProvider } from "../context/ToastContext";

type Options = RenderOptions & {
  route?: string;
  /** Match path so `useParams` works (e.g. `/invoices/:id`). */
  path?: string;
  withRoutes?: boolean;
};

export function renderWithApp(ui: ReactElement, options: Options = {}) {
  const { route = "/", path, withRoutes = false, ...renderOptions } = options;

  function Wrapper({ children }: { children: ReactNode }) {
    const inner = path ? (
      <Routes>
        <Route path={path} element={children} />
      </Routes>
    ) : withRoutes ? (
      <Routes>
        <Route path="*" element={children} />
      </Routes>
    ) : (
      children
    );

    return (
      <MemoryRouter initialEntries={[route]}>
        <ToastProvider>{inner}</ToastProvider>
      </MemoryRouter>
    );
  }

  return render(ui, { wrapper: Wrapper, ...renderOptions });
}

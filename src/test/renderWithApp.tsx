import { render, type RenderOptions } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import type { ReactElement, ReactNode } from "react";
import { ToastProvider } from "../context/ToastContext";

type Options = RenderOptions & {
  route?: string;
  withRoutes?: boolean;
};

export function renderWithApp(ui: ReactElement, options: Options = {}) {
  const { route = "/", withRoutes = false, ...renderOptions } = options;

  function Wrapper({ children }: { children: ReactNode }) {
    return (
      <MemoryRouter initialEntries={[route]}>
        <ToastProvider>
          {withRoutes ? (
            <Routes>
              <Route path="*" element={children} />
            </Routes>
          ) : (
            children
          )}
        </ToastProvider>
      </MemoryRouter>
    );
  }

  return render(ui, { wrapper: Wrapper, ...renderOptions });
}

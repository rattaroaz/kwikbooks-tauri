/** Shared loading placeholder for list/detail pages. */
export function PageLoading({ label = "Loading…" }: { label?: string }) {
  return (
    <div className="kb-page" aria-busy="true">
      <p className="kb-muted">{label}</p>
      <div className="kb-skeleton-stack" aria-hidden="true">
        <div className="kb-skeleton-line kb-skeleton-line-lg" />
        <div className="kb-skeleton-line" />
        <div className="kb-skeleton-line kb-skeleton-line-sm" />
      </div>
    </div>
  );
}

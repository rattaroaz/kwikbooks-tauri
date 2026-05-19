import { Link } from "react-router-dom";

const steps = [
  {
    title: "Company & numbering",
    body: "Set your legal name and document numbering hints in Settings.",
    to: "/settings",
    toLabel: "Open Settings",
  },
  {
    title: "Chart of accounts",
    body: "Review the default ledger (cash, AR, AP, equity, revenue, expense).",
    to: "/accounts",
    toLabel: "Chart of accounts",
  },
  {
    title: "People you bill and pay",
    body: "Add at least one customer and vendor before invoices and bills.",
    to: "/customers",
    toLabel: "Customers",
  },
];

export function WelcomePage() {
  return (
    <div className="kb-page">
      <h1>Getting started</h1>
      <p className="kb-muted">
        Single-company local books seeded with a minimal chart of accounts. Use
        this checklist if you&apos;re opening Kwikbooks for the first time.
      </p>
      <div className="kb-onboarding">
        <h2>Optional: opening balances</h2>
        <p className="kb-muted">
          For now, balance-sheet opening balances live as manual journals (same
          as many desktop accounting apps). Phase 10+ could add a dedicated
          wizard.
        </p>
        <h2>MVP checklist</h2>
        <ol className="kb-onboarding-list">
          {steps.map((s) => (
            <li key={s.title}>
              <strong>{s.title}</strong>
              <p className="kb-muted">{s.body}</p>
              <Link to={s.to} className="kb-button-secondary">
                {s.toLabel}
              </Link>
            </li>
          ))}
          <li>
            <strong>First sale / purchase</strong>
            <p className="kb-muted">
              Create drafts, mark invoices sent before posting GL, bills as open,
              then post from the detail screen.
            </p>
            <div className="kb-onboarding-actions">
              <Link to="/invoices/new" className="kb-button-secondary">
                New invoice
              </Link>
              <Link to="/bills/new" className="kb-button-secondary">
                New bill
              </Link>
            </div>
          </li>
        </ol>
      </div>
      <p className="kb-hint kb-muted">
        Keyboard (Alt+digit): 1 dashboard · 2 accounts · 3 customers · 4 vendors
        · 5 invoices · 6 bills · 7 register · 8 reports · 9 settings · 0 this
        page. Receive payment and pay vendor are in the sidebar.
      </p>
      <Link to="/" className="kb-inline-link">
        Back to Dashboard
      </Link>
    </div>
  );
}

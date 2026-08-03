import type { CheckLayout } from "../lib/checkStock";
import { amountMinorToWords } from "../lib/checkAmountWords";
import { currencyMajorLabel, formatMoneyMinor } from "../lib/money";

export type CheckPrintData = {
  companyName: string;
  legalName?: string | null;
  addressLine1?: string | null;
  addressLine2?: string | null;
  city?: string | null;
  region?: string | null;
  postalCode?: string | null;
  payeeName: string;
  paymentDate: string;
  amountMinor: number;
  memo?: string | null;
  checkNumber?: string | null;
  currencyCode?: string;
};

type Props = {
  layout: CheckLayout;
  data: CheckPrintData;
};

function formatDateForCheck(iso: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso.trim());
  if (!m) {
    return iso;
  }
  return `${m[2]}/${m[3]}/${m[1]}`;
}

function companyAddressLines(data: CheckPrintData): string[] {
  const lines: string[] = [];
  if (data.addressLine1?.trim()) {
    lines.push(data.addressLine1.trim());
  }
  if (data.addressLine2?.trim()) {
    lines.push(data.addressLine2.trim());
  }
  const city = data.city?.trim() ?? "";
  const region = data.region?.trim() ?? "";
  const postal = data.postalCode?.trim() ?? "";
  if (city && region && postal) {
    lines.push(`${city}, ${region} ${postal}`);
  } else {
    const locality = [city, region, postal].filter(Boolean).join(", ");
    if (locality) {
      lines.push(locality);
    }
  }
  return lines;
}

function CheckFace({
  data,
  showGuides,
}: {
  data: CheckPrintData;
  showGuides: boolean;
}) {
  const displayName = data.legalName?.trim() || data.companyName;
  const address = companyAddressLines(data);
  const currency = data.currencyCode ?? "USD";
  const amountWords = amountMinorToWords(data.amountMinor, currency);
  const amountFmt = formatMoneyMinor(data.amountMinor, currency);
  const majorLabel = currencyMajorLabel(currency);
  const guide = showGuides ? " kb-check-guides" : "";

  return (
    <div className={`kb-check-face${guide}`}>
      <div className="kb-check-company">
        <div className="kb-check-company-name">{displayName}</div>
        {address.map((line) => (
          <div key={line} className="kb-check-company-line">
            {line}
          </div>
        ))}
      </div>
      <div className="kb-check-number-field">
        {data.checkNumber ? `No. ${data.checkNumber}` : ""}
      </div>
      <div className="kb-check-date-field">
        <span className="kb-check-label">Date</span>
        <span className="kb-check-value">
          {formatDateForCheck(data.paymentDate)}
        </span>
      </div>
      <div className="kb-check-payee-field">
        <span className="kb-check-label">Pay to the order of</span>
        <span className="kb-check-value">{data.payeeName}</span>
      </div>
      <div className="kb-check-amount-num-field">
        <span className="kb-check-value">{amountFmt}</span>
      </div>
      <div className="kb-check-amount-words-field">
        <span className="kb-check-value">{amountWords}</span>
        <span className="kb-check-dollars-suffix">{majorLabel}</span>
      </div>
      <div className="kb-check-memo-field">
        <span className="kb-check-label">Memo</span>
        <span className="kb-check-value">{data.memo?.trim() || ""}</span>
      </div>
      <div className="kb-check-signature-field">
        <span className="kb-check-label">Authorized signature</span>
      </div>
    </div>
  );
}

export function CheckPrintLayout({ layout, data }: Props) {
  const showGuides = layout === "generic";
  const band =
    layout === "generic" || layout === "voucher_top"
      ? "top"
      : layout === "voucher_middle"
        ? "middle"
        : "bottom";

  return (
    <div
      className={`kb-check-page kb-check-band-${band}${showGuides ? " kb-check-page-generic" : ""}`}
      data-testid="check-print-layout"
      data-layout={layout}
    >
      <div className="kb-check-band">
        <CheckFace data={data} showGuides={showGuides} />
      </div>
      <div className="kb-check-stub kb-check-stub-a" aria-hidden="true">
        <div className="kb-check-stub-title">Voucher</div>
        <div>
          {data.payeeName} · {formatMoneyMinor(data.amountMinor, data.currencyCode ?? "USD")}
        </div>
        <div>{data.memo?.trim() || ""}</div>
      </div>
      <div className="kb-check-stub kb-check-stub-b" aria-hidden="true">
        <div className="kb-check-stub-title">Record</div>
        <div>Check {data.checkNumber ?? "—"}</div>
        <div>{formatDateForCheck(data.paymentDate)}</div>
      </div>
    </div>
  );
}

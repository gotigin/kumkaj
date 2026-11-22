/**
 * Kumkaj Cloud shop catalog — shared shape for all four product sites.
 * Prices: docs/CLOUD_PLANS.md (tax exclusive INR). Custom = quote only.
 */
export const CLOUD_CATALOG = [
  {
    id: "free",
    title: "Free forever",
    blurb: "Full local product on your devices. No cloud fee.",
    priceInrPaise: 0,
    quotaGb: null,
    sellable: false,
    quoteOnly: false,
  },
  {
    id: "steady",
    title: "Steady",
    blurb: "Live cloud · 10 GB · lower bandwidth",
    priceInrPaise: 600_000,
    quotaGb: 10,
    sellable: true,
    quoteOnly: false,
  },
  {
    id: "swift",
    title: "Swift",
    blurb: "Live cloud · 25 GB · mid bandwidth",
    priceInrPaise: 1_200_000,
    quotaGb: 25,
    sellable: true,
    quoteOnly: false,
  },
  {
    id: "priority",
    title: "Priority",
    blurb: "Live cloud · 50 GB · highest standard bandwidth",
    priceInrPaise: 2_400_000,
    quotaGb: 50,
    sellable: true,
    quoteOnly: false,
  },
  {
    id: "custom",
    title: "Custom",
    blurb: "Agreed quota & bandwidth. Quote may be below Steady or above Priority.",
    priceInrPaise: null,
    quotaGb: null,
    sellable: false,
    quoteOnly: true,
  },
];

export function formatInrPaise(paise) {
  if (paise == null) return "Quote";
  if (paise === 0) return "₹0";
  return `₹${(paise / 100).toLocaleString("en-IN")}/yr ex-tax`;
}

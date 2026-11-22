import assert from "node:assert/strict";
import { CLOUD_CATALOG, formatInrPaise } from "./catalog.js";

assert.equal(CLOUD_CATALOG.length, 5);
assert.deepEqual(
  CLOUD_CATALOG.map((p) => p.id),
  ["free", "steady", "swift", "priority", "custom"]
);
assert.equal(formatInrPaise(0), "₹0");
assert.equal(formatInrPaise(600_000), "₹6,000/yr ex-tax");
assert.equal(formatInrPaise(null), "Quote");
assert.ok(CLOUD_CATALOG.find((p) => p.id === "steady").sellable);
assert.ok(CLOUD_CATALOG.find((p) => p.id === "custom").quoteOnly);
assert.ok(!CLOUD_CATALOG.find((p) => p.id === "free").sellable);
assert.ok(
  !/\bLAN\b/i.test(CLOUD_CATALOG.find((p) => p.id === "free").blurb),
  "Free forever blurb must not claim LAN until LAN_PILOT_PASSED"
);
console.log("catalog.test.mjs: ok");

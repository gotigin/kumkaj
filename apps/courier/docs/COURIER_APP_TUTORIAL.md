# Day book

> **Scope:** owner-facing tutorial for **Kumkaj Courier only** (booking,
> parcels, carriers). The general books model it teaches — Day book as
> home, money in/out, “they owe us / we owe them”, cash vs bank — is
> Kumkaj-wide and canonical in
> [`BUSINESS_BOOKS.md`](../../../docs/BUSINESS_BOOKS.md) § BOS user
> experience; another app writes its own tutorial in its own words.

The Day book is the first screen you see after unlocking the app in Server Mode.
It answers one question: **where does the business stand today?**

You start your day here, and you end your day here.
The numbers on this screen update by themselves as you work — there is no Refresh button, because there is nothing to refresh. What you see is always the current position.

---

## The strip at the top

Above the Day book heading, a thin strip stays visible on every screen:

```text
Today: 3 · ₹1,250.00
Cash ₹700.00 · UPI ₹400.00 · Card ₹150.00
```

- **Today: 3 · ₹1,250.00** — 3 bookings made today, worth ₹1,250 in total.
- **Cash · UPI · Card** — how today's received money actually came in: cash in the drawer, UPI, and card.

Two more lines appear in this strip only when they are not zero:

- **Account booking … · Parties … outstanding** — how much of today's booking was put on account (not yet paid), and the total all parties owe you right now.
- **Round …** — small rounding differences from today's bills, when any exist.

On a brand-new shop with no bookings yet, all of this simply reads ₹0.00. That is normal — it fills up as you book.

---

## "Today — books at a glance"

This is the heart of the screen. Each line is one number, explained in counter language:

**Money in (sales)**
The total value of today's bookings — every parcel booked today, whether the customer paid at the counter or took it on account.

**Money out — purchases (COGS) · expenses**
Two kinds of money going out, kept separate:

- *Purchases (COGS)* — what the carriers charge you against today's bookings. This is the cost of moving today's parcels.
- *Expenses* — the day's expense vouchers: rent, fuel, packing material, tea — money out that is not tied to one parcel.

**Gross · Net**

- *Gross* = Money in − purchases. What today's bookings earned you before daily expenses.
- *Net* = Gross − expenses. What today actually left with you.

**Account booking · Cash booking**
How today's bookings split by how they were paid for:

- *Account booking* — booked on trust; the party owes you this much more now.
- *Cash booking* — paid at the counter at booking time.

**Cash · UPI · Card**
The money physically received today, by method. Account bookings are **not** counted here — they enter this line only on the day the party actually pays you.

**They owe us**
Everything all parties owe you, across all days — not just today. This is money you have earned but not yet collected.

**We owe them**
Everything you owe carriers and parties, across all days — bills recorded but not yet paid.

**Bank balance**
What your bank account holds, according to the books.

**Bank net today**
Bank money in minus bank money out, for today only. Positive means the bank gained today; negative means it went down.

---

## The five buttons

### Book a parcel

Takes you to the **Booking** screen — the counter desk.

This is where a customer walks in with a parcel: you pick the consignor (or walk-in), weigh the parcel, the rate comes from your rate card, you take cash/UPI/card or put it on the party's account, and the booking is saved with its AWB/docket number.

Use it for: every parcel that comes over the counter.

### Expense / carrier bill

Takes you to the **Pay out** screen — money out.

Two things are recorded here:

- A **carrier bill** — what a carrier charges you for moving parcels. If you are not paying right now, it stays on that carrier's ledger as "we owe them" until you pay.
- An **expense voucher** — daily spending not tied to a parcel: rent, fuel, cartons, salary advances.

Use it for: every rupee that leaves the shop, so the Day book's money-out lines stay true.

### Day close

Takes you to the **Day close** screen — the end-of-day ritual.

You count the cash in the drawer and match it against what the app says should be there, then close the day. Differences are seen the same night, while you still remember the day — not weeks later.

Use it for: the last thing before you shut the shop. Every day, without exception.

### Bank book & P&L

Takes you to the **Reports** screen, opened at the bank book.

Here you see bank entries (deposits, withdrawals, transfers), and the profit & loss view: money in, money out, and what the business earned over a day, a month, or any period.

Use it for: answering "how is the business doing?" — and for matching the app against your actual bank statement.

### Parties (debtors & creditors)

Takes you to the **Parties** screen — everyone you deal with.

One list holds consignors (who send parcels), carriers (who move them), and parties who are both. For each party you see what they owe you (*debtors* — "they owe us") and what you owe them (*creditors* — "we owe them"), kept as two separate ledgers that never mix.

This is also where you add a new party, correct a name or phone number, record a payment received from a party, or pay a carrier.

Use it for: before extending credit to anyone — check what they already owe you first.

---

## Three things worth remembering

1. **This screen is a mirror, not a notebook.** Nothing is typed here. Every number comes from Booking, Pay out, Parties, and the bank book. If a number looks wrong, the fix is made on the screen where the entry was made — the Day book then corrects itself.

2. **"They owe us" and "We owe them" are totals across all time**, while everything above them is today only. A busy day can still leave "they owe us" high if old account customers have not paid — that is the number to chase.

3. **Check this screen before you go home.** Money in, money out, and the drawer should tell one story. If they do not, Day close will show it — better tonight than at month-end.

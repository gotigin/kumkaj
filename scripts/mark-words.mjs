#!/usr/bin/env node
// Mark word boundaries with U+200B ZERO WIDTH SPACE in Thai, Lao, Khmer and
// Burmese language packs (key<TAB>text TSV, `#` comment lines kept as they
// are). These scripts write a sentence without spaces between its words;
// Makepad wraps them at UAX #14 break opportunities (vendor patch
// line_break_uax14), and a zero-width space is one — so with the marks a row
// breaks between words instead of inside one.
//
//   node scripts/mark-words.mjs <lang> <pack.tsv>...        rewrite the packs in place
//   node scripts/mark-words.mjs --check <lang> <pack.tsv>  exit 1 if a pack would change
//
// Word boundaries come from ICU's dictionary word segmenter
// (Intl.Segmenter, granularity "word") — the same one browsers use. A mark
// goes only between two word-like segments that both touch the script's
// letters, so spaces, punctuation, numbers, Latin words and {placeholders}
// are never marked. Existing marks are removed first: re-running is safe.
import { readFileSync, writeFileSync } from "node:fs";

const SCRIPTS = {
  th: /[฀-๿]/u,
  lo: /[຀-໿]/u,
  km: /[ក-៿᧠-᧿]/u,
  my: /[က-႟ꧠ-꧿ꩠ-ꩿ]/u,
};
const ZWSP = "​";
// Never a mark before: a combining mark, or a repetition / abbreviation
// mark (ๆ ฯ ໆ ຯ ៗ), which belong to the word before them.
const NO_MARK_BEFORE = /^[\p{M}ๆฯໆຯៗ]/u;
// Never a mark after: Khmer COENG or the Myanmar stacker (the next letter
// is stacked under this one), a Thai or Lao vowel written before its
// consonant, or a joiner.
const NO_MARK_AFTER = /[្္เแโใไເແໂໃໄ‌‍]$/u;
// A lone consonant is a piece of an unknown (often borrowed) word the
// dictionary split — ໂຫຼ|ດ, ຄ|ລາວ, គ|ម្រោ|ង — so it gets no mark on either
// side. Burmese is left out: a bare consonant is a whole syllable there.
const LONE_CONSONANT = {
  th: /^[ก-ฮ]$/u,
  lo: /^[ກ-ຮໜໝ]$/u,
  km: /^[ក-អ]$/u,
};

export function mark(lang, text) {
  const letter = SCRIPTS[lang];
  const lone = LONE_CONSONANT[lang];
  const plain = text.replaceAll(ZWSP, "");
  const segments = [...new Intl.Segmenter(lang, { granularity: "word" }).segment(plain)];
  let out = "";
  for (let i = 0; i < segments.length; i++) {
    const s = segments[i];
    out += s.segment;
    const next = segments[i + 1];
    if (
      next &&
      s.isWordLike &&
      next.isWordLike &&
      letter.test([...s.segment].at(-1)) &&
      letter.test([...next.segment][0]) &&
      !NO_MARK_BEFORE.test(next.segment) &&
      !NO_MARK_AFTER.test(s.segment) &&
      !(lone && (lone.test(s.segment) || lone.test(next.segment)))
    ) {
      out += ZWSP;
    }
  }
  return out;
}

function markPack(lang, source) {
  return source
    .split("\n")
    .map((line) => {
      if (line === "" || line.startsWith("#")) return line;
      const tab = line.indexOf("\t");
      if (tab < 0) return line;
      // `\n` escapes stay whole: mark each line of the text on its own.
      const text = line.slice(tab + 1).split("\\n").map((part) => mark(lang, part)).join("\\n");
      return line.slice(0, tab + 1) + text;
    })
    .join("\n");
}

const args = process.argv.slice(2);
const check = args[0] === "--check";
if (check) args.shift();
const [lang, ...files] = args;
if (!SCRIPTS[lang] || files.length === 0) {
  console.error("usage: node scripts/mark-words.mjs [--check] <th|lo|km|my> <pack.tsv>...");
  process.exit(2);
}
let stale = 0;
for (const file of files) {
  const before = readFileSync(file, "utf8");
  const after = markPack(lang, before);
  const changed = before.split("\n").filter((line, i) => line !== after.split("\n")[i]).length;
  if (check) {
    if (changed) {
      console.error(`${file}: ${changed} rows need word marks`);
      stale++;
    }
  } else {
    if (changed) writeFileSync(file, after);
    console.log(`${file}: ${changed} rows marked`);
  }
}
process.exit(stale ? 1 : 0);

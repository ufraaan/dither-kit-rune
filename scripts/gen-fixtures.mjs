// Regenerates the engine fixtures from the pinned Dither Kit sources.
//
//   node scripts/gen-fixtures.mjs <path-to-pinned-dither-kit>/registry/dither-kit
//
// The script concatenates palette.ts + dither-paint.ts, shims just enough of
// CanvasRenderingContext2D to run paintColumn, and writes raw RGBA buffers into
// crates/dither-core/tests/fixtures/. Run it only when the upstream pin moves.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const upstream = process.argv[2];
if (!upstream) {
  console.error("usage: node scripts/gen-fixtures.mjs <dither-kit registry dir>");
  process.exit(2);
}

const strip = (src) =>
  src
    .replace(/^import[\s\S]*?from\s+["'][^"']+["']\s*$/gm, "")
    .replace(/^export\s+/gm, "");

const prelude = `
const __out = {};
const __enc = (data) => Buffer.from(data).toString("base64");
function makeCanvas(w, h) {
  const data = new Uint8ClampedArray(w * h * 4);
  const ctx = {
    fillStyle: "rgba(0,0,0,0)",
    canvas: { width: w, height: h },
    fillRect(x, y, rw, rh) {
      const m = /rgba\\(([^)]+)\\)/.exec(String(this.fillStyle));
      const [r, g, b, a] = m[1].split(",").map(Number);
      for (let py = y; py < y + rh; py++) {
        for (let px = x; px < x + rw; px++) {
          if (px < 0 || py < 0 || px >= w || py >= h) continue;
          const i = (py * w + px) * 4;
          const da = data[i + 3] / 255;
          const oa = a + da * (1 - a);
          if (oa <= 0) continue;
          for (let c = 0; c < 3; c++) {
            const sc = [r, g, b][c];
            data[i + c] = Math.round((sc * a + data[i + c] * da * (1 - a)) / oa);
          }
          data[i + 3] = Math.round(oa * 255);
        }
      }
    },
  };
  return { ctx, data };
}
`;

const scenarios = `
const CASES = [
  ["gradient", 8, 12, 1, 1.4, 9.6, { variant: "gradient", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
  ["gradient-hover", 8, 12, 2, 2.2, 10.4, { variant: "gradient", intensity: 0.7, dim: 1, stacked: false, sparse: 0 }],
  ["dotted", 8, 12, 3, 1.6, 10.2, { variant: "dotted", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
  ["hatched", 8, 12, 4, 2.6, 11.2, { variant: "hatched", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
  ["solid", 8, 12, 5, 3.4, 11.6, { variant: "solid", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
  ["stacked", 8, 12, 6, 1.2, 8.8, { variant: "gradient", intensity: 0, dim: 1, stacked: true, sparse: 0 }],
  ["dimmed", 8, 12, 0, 2.4, 9.2, { variant: "gradient", intensity: 0, dim: 0.3, stacked: false, sparse: 0.14 }],
  ["zero-depth", 8, 12, 7, 5.6, 5.6, { variant: "gradient", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
  ["negative-top", 8, 12, 2, -2.2, 4.4, { variant: "gradient", intensity: 0, dim: 1, stacked: false, sparse: 0 }],
];
for (const [name, cols, rows, x, top, floor, opts] of CASES) {
  const { ctx, data } = makeCanvas(cols, rows);
  paintColumn(ctx, x, top, floor, PALETTE.blue, opts);
  __out[name] = { cols, rows, x, top, floor, opts, rgba: __enc(data) };
}
__out.resample = {
  a: resample([0, 10, 30, 5], 7),
  b: resample([], 4),
  c: resample([7], 5),
};
__out.backing = [backingSize(560, 220), backingSize(10, 10), backingSize(4000, 4000)];
__out.rng = {
  fnv: ["", "ada", "Ada Lovelace", "jürgen", "🦀 rust"].map(fnv1a),
  xorshift1: (() => { const r = xorshift32(1); return Array.from({ length: 5 }, r); })(),
  xorshift0: (() => { const r = xorshift32(0); return Array.from({ length: 5 }, r); })(),
  hue: [0, 60, 120, 240, 359.5, -30].map((h) => hueFill(h)),
};
console.log(JSON.stringify(__out));
`;

// pixel.ts re-declares clamp01; dither-paint.ts's copy is the one paintColumn uses.
const pixel = strip(readFileSync(join(upstream, "pixel.ts"), "utf8")).replace(
  /^const clamp01 =.*$/m,
  "",
);

const source =
  prelude +
  strip(readFileSync(join(upstream, "palette.ts"), "utf8")) +
  "\n" +
  pixel +
  "\n" +
  strip(readFileSync(join(upstream, "dither-paint.ts"), "utf8")) +
  scenarios;

const tmp = join(tmpdir(), `dither-fixtures-${process.pid}.ts`);
writeFileSync(tmp, source);
const output = execFileSync(process.execPath, [tmp], { encoding: "utf8" });
const parsed = JSON.parse(output);

const fixtures = join(root, "crates/dither-core/tests/fixtures");
mkdirSync(fixtures, { recursive: true });
const paints = {};
for (const [name, value] of Object.entries(parsed)) {
  if (!value.rgba) continue;
  writeFileSync(join(fixtures, `${name}.rgba`), Buffer.from(value.rgba, "base64"));
  paints[name] = { cols: value.cols, rows: value.rows, x: value.x, top: value.top, floor: value.floor, opts: value.opts };
}
writeFileSync(
  join(fixtures, "expected.json"),
  JSON.stringify({ rng: parsed.rng, resample: parsed.resample, backing: parsed.backing }, null, 2) + "\n",
);
console.log(`wrote ${Object.keys(paints).length} paint fixtures to ${fixtures}`);

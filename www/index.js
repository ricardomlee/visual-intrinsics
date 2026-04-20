import { M128i, M256i, M512i } from "visual-intrinsics";

// ── Register-type state ───────────────────────────────────────────────────────
let regType = "m128i";  // "m128i" | "m256i" | "m512i"

const REG_CLASS = { m128i: M128i, m256i: M256i, m512i: M512i };
const REG_BITS  = { m128i: 128,   m256i: 256,   m512i: 512   };
const REG_TAG   = { m128i: "__m128i", m256i: "__m256i", m512i: "__m512i" };

const DEMO_HEX = {
  m128i: {
    a: "0x00ff00ff0f0f0f0faaaaaaaa12345678",
    b: "0xff00ff00f0f0f0f05555555587654321",
  },
  m256i: {
    a: "0x00ff00ff0f0f0f0faaaaaaaa1234567800ff00ff0f0f0f0faaaaaaaa12345678",
    b: "0xff00ff00f0f0f0f05555555587654321ff00ff00f0f0f0f05555555587654321",
  },
  m512i: {
    a: "0x00ff00ff0f0f0f0faaaaaaaa1234567800ff00ff0f0f0f0faaaaaaaa1234567800ff00ff0f0f0f0faaaaaaaa1234567800ff00ff0f0f0f0faaaaaaaa12345678",
    b: "0xff00ff00f0f0f0f05555555587654321ff00ff00f0f0f0f05555555587654321ff00ff00f0f0f0f05555555587654321ff00ff00f0f0f0f05555555587654321",
  },
};

let regA, regB, regR;
let lastOp = "—";
let prevResultBits = null;

const views = { a: "epi8", b: "epi8", r: "epi8" };

// ── Helpers ───────────────────────────────────────────────────────────────────
function newReg() { return REG_CLASS[regType].new(); }

function bitsPerLane(view) {
  return { epi8: 8, epu8: 8, epi16: 16, epu16: 16,
           epi32: 32, epu32: 32, epi64: 64 }[view] ?? REG_BITS[regType];
}

// ── Build bit grid ────────────────────────────────────────────────────────────
function buildGrid(id) {
  const el = document.getElementById(id);
  el.innerHTML = "";
  const total = REG_BITS[regType];
  for (let i = 0; i < total; i++) {
    const d = document.createElement("div");
    d.className = "bit off";
    d.dataset.idx = i;
    d.title = `bit ${i}`;
    el.appendChild(d);
  }
}

// ── Render one register ───────────────────────────────────────────────────────
function renderReg(key, reg) {
  const view     = views[key];
  const bits     = reg.get_bits();
  const totalBits = REG_BITS[regType];
  const bpl      = bitsPerLane(view);

  // bit grid
  const cells = document.getElementById("grid-" + key).children;
  for (let i = 0; i < totalBits; i++) {
    const cell = cells[i];
    const newBit = bits[i];
    const wasChanged = key === "r" && prevResultBits !== null && prevResultBits[i] !== newBit;

    cell.className = "bit " + (newBit === "1" ? "on" : "off");
    if (view === "bits") {
      cell.removeAttribute("data-lane");
      cell.title = `bit ${i}`;
    } else {
      const lane = Math.floor(i / bpl) % 8;
      cell.dataset.lane = lane;
      cell.title = `bit ${i} · lane ${lane}`;
    }
    if (wasChanged) {
      cell.classList.remove("changed");
      void cell.offsetWidth;
      cell.classList.add("changed");
      cell.addEventListener("animationend", () => cell.classList.remove("changed"), { once: true });
    }
  }
  if (key === "r") prevResultBits = Array.from(bits);

  // lane value table
  const table = document.getElementById("lanes-" + key);
  table.innerHTML = "";

  let vals;
  switch (view) {
    case "epi8":  vals = JSON.parse(reg.get_epi8());  break;
    case "epu8":  vals = JSON.parse(reg.get_epu8());  break;
    case "epi16": vals = JSON.parse(reg.get_epi16()); break;
    case "epu16": vals = JSON.parse(reg.get_epu16()); break;
    case "epi32": vals = JSON.parse(reg.get_epi32()); break;
    case "epu32": vals = JSON.parse(reg.get_epu32()); break;
    case "epi64": vals = JSON.parse(reg.get_epi64()).map(String); break;
    default:      vals = [reg.to_hex()]; break;
  }

  const nibblesPerLane = bpl / 4;
  for (let li = vals.length - 1; li >= 0; li--) {
    const row    = document.createElement("div");
    row.className = "lane-row";

    const idxEl = document.createElement("span");
    idxEl.className = "lane-idx";
    idxEl.textContent = "[" + li + "]";

    const valEl = document.createElement("span");
    valEl.className = "lane-val";
    valEl.textContent = vals[li];

    const hexEl = document.createElement("span");
    hexEl.className = "lane-hex";
    if (view !== "bits") {
      let v = BigInt(vals[li]);
      if (v < 0n) v += (1n << BigInt(bpl));
      hexEl.textContent = "0x" + v.toString(16).toUpperCase().padStart(nibblesPerLane, "0");
    }

    row.appendChild(idxEl);
    row.appendChild(valEl);
    row.appendChild(hexEl);
    table.appendChild(row);
  }

  if (key === "a") document.getElementById("hex-a").value = reg.to_hex();
  if (key === "b") document.getElementById("hex-b").value = reg.to_hex();
  if (key === "r") document.getElementById("result-hex").textContent = reg.to_hex();
}

function renderAll() {
  renderReg("a", regA);
  renderReg("b", regB);
  renderReg("r", regR);
  document.getElementById("result-op-label").textContent = lastOp;

  // Trigger result panel fade-in animation
  const resultPanel = document.querySelector(".result-panel");
  resultPanel.classList.remove("updated");
  void resultPanel.offsetWidth; // force reflow
  resultPanel.classList.add("updated");
}

// ── Register-type switching ───────────────────────────────────────────────────
function switchRegType(type) {
  regType = type;
  prevResultBits = null;
  const tag = REG_TAG[type];
  document.getElementById("tag-a").textContent = tag;
  document.getElementById("tag-b").textContent = tag;

  buildGrid("grid-a");
  buildGrid("grid-b");
  buildGrid("grid-r");

  const Cls = REG_CLASS[type];
  const demo = DEMO_HEX[type];
  regA = Cls.from_hex(demo.a);
  regB = Cls.from_hex(demo.b);
  regR = Cls.new();
  lastOp = "—";
  renderAll();
}

// ── View-tab switching ────────────────────────────────────────────────────────
function setupTabs(tabsId, key) {
  document.getElementById(tabsId).addEventListener("click", e => {
    if (!e.target.dataset.view) return;
    e.currentTarget.querySelectorAll("button").forEach(b => b.classList.remove("active"));
    e.target.classList.add("active");
    views[key] = e.target.dataset.view;
    renderReg(key, key === "a" ? regA : key === "b" ? regB : regR);
  });
}

// ── Hex input helpers ─────────────────────────────────────────────────────────
function trySetFromHex(inputId, key) {
  const raw = document.getElementById(inputId).value.trim();
  try {
    const reg = REG_CLASS[regType].from_hex(raw || "0");
    if (key === "a") regA = reg; else regB = reg;
    renderReg(key, key === "a" ? regA : regB);
  } catch (e) {
    alert("Invalid hex: " + e);
  }
}

// ── Operations ────────────────────────────────────────────────────────────────
function applyOp(op) {
  const bits  = (parseInt(document.getElementById("shift-bits").value) || 0) >>> 0;
  const count = (parseInt(document.getElementById("lane-count").value) || 0) >>> 0;
  const imm8  = (parseInt(document.getElementById("imm8-val").value)   || 0) & 0xff;
  const pfx   = regType === "m128i" ? "_mm_" : regType === "m256i" ? "_mm256_" : "_mm512_";

  switch (op) {
    // ── Bitwise
    case "and":    regR = regA.and(regB);    lastOp = pfx + "and_si"; break;
    case "or":     regR = regA.or(regB);     lastOp = pfx + "or_si";  break;
    case "xor":    regR = regA.xor(regB);    lastOp = pfx + "xor_si"; break;
    case "andnot": regR = regA.andnot(regB); lastOp = pfx + "andnot_si"; break;
    case "not-a":  regR = regA.not();        lastOp = "NOT A"; break;
    case "not-b":  regR = regB.not();        lastOp = "NOT B"; break;

    // ── Add
    case "add_epi8":  regR = regA.add_epi8(regB);  lastOp = pfx + "add_epi8";  break;
    case "add_epi16": regR = regA.add_epi16(regB); lastOp = pfx + "add_epi16"; break;
    case "add_epi32": regR = regA.add_epi32(regB); lastOp = pfx + "add_epi32"; break;
    case "add_epi64": regR = regA.add_epi64(regB); lastOp = pfx + "add_epi64"; break;

    // ── Sub
    case "sub_epi8":  regR = regA.sub_epi8(regB);  lastOp = pfx + "sub_epi8";  break;
    case "sub_epi16": regR = regA.sub_epi16(regB); lastOp = pfx + "sub_epi16"; break;
    case "sub_epi32": regR = regA.sub_epi32(regB); lastOp = pfx + "sub_epi32"; break;
    case "sub_epi64": regR = regA.sub_epi64(regB); lastOp = pfx + "sub_epi64"; break;

    // ── Saturating add
    case "adds_epi8":  regR = regA.adds_epi8(regB);  lastOp = pfx + "adds_epi8";  break;
    case "adds_epi16": regR = regA.adds_epi16(regB); lastOp = pfx + "adds_epi16"; break;
    case "adds_epu8":  regR = regA.adds_epu8(regB);  lastOp = pfx + "adds_epu8";  break;
    case "adds_epu16": regR = regA.adds_epu16(regB); lastOp = pfx + "adds_epu16"; break;

    // ── Saturating sub
    case "subs_epi8":  regR = regA.subs_epi8(regB);  lastOp = pfx + "subs_epi8";  break;
    case "subs_epi16": regR = regA.subs_epi16(regB); lastOp = pfx + "subs_epi16"; break;
    case "subs_epu8":  regR = regA.subs_epu8(regB);  lastOp = pfx + "subs_epu8";  break;
    case "subs_epu16": regR = regA.subs_epu16(regB); lastOp = pfx + "subs_epu16"; break;

    // ── Multiply
    case "mullo_epi16": regR = regA.mullo_epi16(regB); lastOp = pfx + "mullo_epi16"; break;
    case "mulhi_epi16": regR = regA.mulhi_epi16(regB); lastOp = pfx + "mulhi_epi16"; break;
    case "mullo_epi32": regR = regA.mullo_epi32(regB); lastOp = pfx + "mullo_epi32"; break;

    // ── Abs
    case "abs_epi8":  regR = regA.abs_epi8();  lastOp = pfx + "abs_epi8";  break;
    case "abs_epi16": regR = regA.abs_epi16(); lastOp = pfx + "abs_epi16"; break;
    case "abs_epi32": regR = regA.abs_epi32(); lastOp = pfx + "abs_epi32"; break;

    // ── Max signed
    case "max_epi8":  regR = regA.max_epi8(regB);  lastOp = pfx + "max_epi8";  break;
    case "max_epi16": regR = regA.max_epi16(regB); lastOp = pfx + "max_epi16"; break;
    case "max_epi32": regR = regA.max_epi32(regB); lastOp = pfx + "max_epi32"; break;

    // ── Min signed
    case "min_epi8":  regR = regA.min_epi8(regB);  lastOp = pfx + "min_epi8";  break;
    case "min_epi16": regR = regA.min_epi16(regB); lastOp = pfx + "min_epi16"; break;
    case "min_epi32": regR = regA.min_epi32(regB); lastOp = pfx + "min_epi32"; break;

    // ── Max unsigned
    case "max_epu8":  regR = regA.max_epu8(regB);  lastOp = pfx + "max_epu8";  break;
    case "max_epu16": regR = regA.max_epu16(regB); lastOp = pfx + "max_epu16"; break;
    case "max_epu32": regR = regA.max_epu32(regB); lastOp = pfx + "max_epu32"; break;

    // ── Min unsigned
    case "min_epu8":  regR = regA.min_epu8(regB);  lastOp = pfx + "min_epu8";  break;
    case "min_epu16": regR = regA.min_epu16(regB); lastOp = pfx + "min_epu16"; break;
    case "min_epu32": regR = regA.min_epu32(regB); lastOp = pfx + "min_epu32"; break;

    // ── Compare eq
    case "cmpeq_epi8":  regR = regA.cmpeq_epi8(regB);  lastOp = pfx + "cmpeq_epi8";  break;
    case "cmpeq_epi16": regR = regA.cmpeq_epi16(regB); lastOp = pfx + "cmpeq_epi16"; break;
    case "cmpeq_epi32": regR = regA.cmpeq_epi32(regB); lastOp = pfx + "cmpeq_epi32"; break;
    case "cmpeq_epi64": regR = regA.cmpeq_epi64(regB); lastOp = pfx + "cmpeq_epi64"; break;

    // ── Compare gt
    case "cmpgt_epi8":  regR = regA.cmpgt_epi8(regB);  lastOp = pfx + "cmpgt_epi8";  break;
    case "cmpgt_epi16": regR = regA.cmpgt_epi16(regB); lastOp = pfx + "cmpgt_epi16"; break;
    case "cmpgt_epi32": regR = regA.cmpgt_epi32(regB); lastOp = pfx + "cmpgt_epi32"; break;
    case "cmpgt_epi64": regR = regA.cmpgt_epi64(regB); lastOp = pfx + "cmpgt_epi64"; break;

    // ── Horizontal
    case "hadd_epi16": regR = regA.hadd_epi16(regB); lastOp = pfx + "hadd_epi16"; break;
    case "hadd_epi32": regR = regA.hadd_epi32(regB); lastOp = pfx + "hadd_epi32"; break;
    case "hsub_epi16": regR = regA.hsub_epi16(regB); lastOp = pfx + "hsub_epi16"; break;
    case "hsub_epi32": regR = regA.hsub_epi32(regB); lastOp = pfx + "hsub_epi32"; break;

    // ── Per-lane shifts
    case "slli_epi16": regR = regA.slli_epi16(count); lastOp = pfx + `slli_epi16(A,${count})`; break;
    case "srli_epi16": regR = regA.srli_epi16(count); lastOp = pfx + `srli_epi16(A,${count})`; break;
    case "srai_epi16": regR = regA.srai_epi16(count); lastOp = pfx + `srai_epi16(A,${count})`; break;
    case "slli_epi32": regR = regA.slli_epi32(count); lastOp = pfx + `slli_epi32(A,${count})`; break;
    case "srli_epi32": regR = regA.srli_epi32(count); lastOp = pfx + `srli_epi32(A,${count})`; break;
    case "srai_epi32": regR = regA.srai_epi32(count); lastOp = pfx + `srai_epi32(A,${count})`; break;
    case "slli_epi64": regR = regA.slli_epi64(count); lastOp = pfx + `slli_epi64(A,${count})`; break;
    case "srli_epi64": regR = regA.srli_epi64(count); lastOp = pfx + `srli_epi64(A,${count})`; break;

    // ── Full-register shifts
    case "shl": regR = regA.shift_left_bits(bits);  lastOp = `SHL A by ${bits} bits`; break;
    case "shr": regR = regA.shift_right_bits(bits); lastOp = `SHR A by ${bits} bits`; break;

    // ── Unpack low
    case "unpacklo_epi8":  regR = regA.unpacklo_epi8(regB);  lastOp = pfx + "unpacklo_epi8";  break;
    case "unpacklo_epi16": regR = regA.unpacklo_epi16(regB); lastOp = pfx + "unpacklo_epi16"; break;
    case "unpacklo_epi32": regR = regA.unpacklo_epi32(regB); lastOp = pfx + "unpacklo_epi32"; break;
    case "unpacklo_epi64": regR = regA.unpacklo_epi64(regB); lastOp = pfx + "unpacklo_epi64"; break;

    // ── Unpack high
    case "unpackhi_epi8":  regR = regA.unpackhi_epi8(regB);  lastOp = pfx + "unpackhi_epi8";  break;
    case "unpackhi_epi16": regR = regA.unpackhi_epi16(regB); lastOp = pfx + "unpackhi_epi16"; break;
    case "unpackhi_epi32": regR = regA.unpackhi_epi32(regB); lastOp = pfx + "unpackhi_epi32"; break;
    case "unpackhi_epi64": regR = regA.unpackhi_epi64(regB); lastOp = pfx + "unpackhi_epi64"; break;

    // ── Pack
    case "packs_epi16":  regR = regA.packs_epi16(regB);  lastOp = pfx + "packs_epi16";  break;
    case "packs_epi32":  regR = regA.packs_epi32(regB);  lastOp = pfx + "packs_epi32";  break;
    case "packus_epi16": regR = regA.packus_epi16(regB); lastOp = pfx + "packus_epi16"; break;
    case "packus_epi32": regR = regA.packus_epi32(regB); lastOp = pfx + "packus_epi32"; break;

    // ── Shuffle / align / blend
    case "shuffle_epi32": regR = regA.shuffle_epi32(imm8);         lastOp = pfx + `shuffle_epi32(A,0x${imm8.toString(16).toUpperCase().padStart(2,"0")})`; break;
    case "shuffle_epi8":  regR = regA.shuffle_epi8(regB);          lastOp = pfx + "shuffle_epi8(A,B)";         break;
    case "alignr_epi8":   regR = regA.alignr_epi8(regB, imm8);     lastOp = pfx + `alignr_epi8(A,B,${imm8})`; break;
    case "blendv_epi8":   regR = regA.blendv_epi8(regB, regB);     lastOp = pfx + "blendv_epi8(A,B,B)";       break;

    // ── Copy
    case "a-to-b":      regB = regA.clone_reg(); lastOp = "A → B"; break;
    case "b-to-a":      regA = regB.clone_reg(); lastOp = "B → A"; break;
    case "result-to-a": regA = regR.clone_reg(); lastOp = "Result → A"; break;
    case "result-to-b": regB = regR.clone_reg(); lastOp = "Result → B"; break;
  }
  renderAll();
}

// ── Init ──────────────────────────────────────────────────────────────────────
buildGrid("grid-a");
buildGrid("grid-b");
buildGrid("grid-r");

setupTabs("tabs-a", "a");
setupTabs("tabs-b", "b");
setupTabs("tabs-r", "r");

// Register-type selector
document.querySelector(".regtype-sel").addEventListener("click", e => {
  const type = e.target.dataset.type;
  if (!type || type === regType) return;
  e.currentTarget.querySelectorAll("button").forEach(b => b.classList.remove("active"));
  e.target.classList.add("active");
  switchRegType(type);
});

// Hex set
document.getElementById("btn-set-a").addEventListener("click", () => trySetFromHex("hex-a", "a"));
document.getElementById("btn-set-b").addEventListener("click", () => trySetFromHex("hex-b", "b"));
document.getElementById("hex-a").addEventListener("keydown", e => { if (e.key === "Enter") trySetFromHex("hex-a", "a"); });
document.getElementById("hex-b").addEventListener("keydown", e => { if (e.key === "Enter") trySetFromHex("hex-b", "b"); });

// Clear
document.getElementById("btn-clear-a").addEventListener("click", () => { regA = newReg(); renderReg("a", regA); });
document.getElementById("btn-clear-b").addEventListener("click", () => { regB = newReg(); renderReg("b", regB); });

// Operation buttons
document.querySelectorAll(".op-btn[data-op]").forEach(btn => {
  btn.addEventListener("click", () => applyOp(btn.dataset.op));
});

// Seed demo values
regA = M128i.from_hex(DEMO_HEX.m128i.a);
regB = M128i.from_hex(DEMO_HEX.m128i.b);
regR = M128i.new();
renderAll();

// ── Accordion ────────────────────────────────────────────────────────────────
(function initAccordion() {
  function loadState() {
    try { return JSON.parse(localStorage.getItem("vi-accordion") || "{}"); } catch (e) { console.error("vi-accordion load:", e); return {}; }
  }
  function saveState(s) {
    try { localStorage.setItem("vi-accordion", JSON.stringify(s)); } catch (e) { console.error("vi-accordion save:", e); }
  }

  const state = loadState();

  // Top-level category toggles
  document.querySelectorAll(".op-cat").forEach(cat => {
    const id = cat.id;
    if (id && id in state) cat.classList.toggle("open", state[id]);
    cat.querySelector(".op-cat-header").addEventListener("click", () => {
      cat.classList.toggle("open");
      if (id) { const s = loadState(); s[id] = cat.classList.contains("open"); saveState(s); }
    });
  });

  // Sub-group collapsible labels
  document.querySelectorAll(".op-group").forEach(group => {
    const grpId = group.dataset.grpId;
    if (grpId && grpId in state) group.classList.toggle("collapsed", state[grpId]);
    group.querySelector(".op-group-label").addEventListener("click", () => {
      group.classList.toggle("collapsed");
      if (grpId) { const s = loadState(); s[grpId] = group.classList.contains("collapsed"); saveState(s); }
    });
  });
})();

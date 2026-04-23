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
let laneEditMode = false;

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
      const lane = Math.floor(i / bpl);
      const styledLane = lane % 8;
      cell.dataset.lane = styledLane;
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

    let valEl;
    if (laneEditMode && key !== "r") {
      valEl = document.createElement("input");
      valEl.type = "text";
      valEl.className = "lane-val lane-val-input";
      valEl.value = vals[li];
      valEl.title = "Edit lane " + li + " value; press Enter to apply";
      valEl.setAttribute("aria-label", "Edit register " + key + " " + view + " lane " + li + " value");
      // Capture li and view at closure time
      const capturedLi = li;
      const capturedView = view;
      let editCancelled = false;
      valEl.addEventListener("keydown", e => {
        if (e.key === "Enter")  { commitLaneEdit(key, capturedView, capturedLi, valEl.value); e.preventDefault(); }
        if (e.key === "Escape") {
          editCancelled = true;
          e.preventDefault();
          renderReg(key, key === "a" ? regA : regB);
        }
      });
      valEl.addEventListener("blur", () => {
        if (editCancelled) return;
        commitLaneEdit(key, capturedView, capturedLi, valEl.value);
      });
    } else {
      valEl = document.createElement("span");
      valEl.className = "lane-val";
      valEl.textContent = vals[li];
    }

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

// ── Editable lane values ──────────────────────────────────────────────────────
function commitLaneEdit(key, view, laneIdx, newValueStr) {
  const reg = key === "a" ? regA : regB;
  const bpl = bitsPerLane(view);
  const totalBits = REG_BITS[regType];
  const numLanes = totalBits / bpl;
  const nibblesPerLane = bpl / 4;

  // Fetch current lane values fresh from the register
  let vals;
  switch (view) {
    case "epi8":  vals = JSON.parse(reg.get_epi8());  break;
    case "epu8":  vals = JSON.parse(reg.get_epu8());  break;
    case "epi16": vals = JSON.parse(reg.get_epi16()); break;
    case "epu16": vals = JSON.parse(reg.get_epu16()); break;
    case "epi32": vals = JSON.parse(reg.get_epi32()); break;
    case "epu32": vals = JSON.parse(reg.get_epu32()); break;
    case "epi64": vals = JSON.parse(reg.get_epi64()).map(String); break;
    default: return; // "bits" view is not editable lane-by-lane
  }

  // Parse the new value, restoring the display on error
  let newVal;
  try {
    const trimmed = newValueStr.trim();
    if (view === "epi64") {
      newVal = BigInt(trimmed);
      if (newVal < -(1n << 63n) || newVal > (1n << 63n) - 1n) throw new Error("out of range");
    } else {
      newVal = parseInt(trimmed, 10);
      if (isNaN(newVal)) throw new Error("not a number");
    }
  } catch (_) {
    renderReg(key, reg);
    return;
  }

  // Skip reconstruction if the value didn't actually change
  if (String(newVal) === String(vals[laneIdx])) return;

  vals[laneIdx] = newVal;

  // Reconstruct the full register hex (big-endian: MSB lane first)
  let hexStr = "";
  for (let i = numLanes - 1; i >= 0; i--) {
    let v = BigInt(vals[i]);
    if (v < 0n) v += (1n << BigInt(bpl));
    hexStr += v.toString(16).padStart(nibblesPerLane, "0");
  }

  try {
    const newRegObj = REG_CLASS[regType].from_hex("0x" + hexStr);
    if (key === "a") { regA = newRegObj; renderReg("a", regA); }
    else              { regB = newRegObj; renderReg("b", regB); }
  } catch (_) {
    renderReg(key, reg);
  }
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

// Edit-lanes toggle
document.getElementById("lane-edit-toggle").addEventListener("click", function () {
  laneEditMode = !laneEditMode;
  this.classList.toggle("active", laneEditMode);
  this.setAttribute("aria-pressed", laneEditMode ? "true" : "false");
  renderReg("a", regA);
  renderReg("b", regB);
});

// Seed demo values
regA = M128i.from_hex(DEMO_HEX.m128i.a);
regB = M128i.from_hex(DEMO_HEX.m128i.b);
regR = M128i.new();
renderAll();

// ── Save result hex to clipboard ──────────────────────────────────────────────
document.getElementById("btn-save-hex").addEventListener("click", () => {
  const hex = document.getElementById("result-hex").textContent;
  const btn = document.getElementById("btn-save-hex");
  navigator.clipboard.writeText(hex).then(() => {
    btn.textContent = "✓";
    setTimeout(() => { btn.textContent = "💾"; }, 1000);
  }).catch(() => { alert("Copy failed — " + hex); });
});

// ── Multi-level operations navigation ────────────────────────────────────────
(function initOpNav() {
  function loadState() {
    try { return JSON.parse(localStorage.getItem("vi-opnav") || "{}"); } catch (e) { return {}; }
  }
  function saveState(s) {
    try { localStorage.setItem("vi-opnav", JSON.stringify(s)); } catch (e) {}
  }

  const state = loadState();

  // Level 1 — restore active category from localStorage
  const level1 = document.getElementById("op-level1");
  const activeCat = (state["_cat"] && document.getElementById("op-level2-" + state["_cat"]))
    ? state["_cat"] : "utilities";
  level1.querySelectorAll(".op-l1-btn").forEach(btn => {
    btn.classList.toggle("active", btn.dataset.cat === activeCat);
  });
  document.querySelectorAll(".op-level2").forEach(p => {
    p.hidden = (p.id !== "op-level2-" + activeCat);
  });

  // Level 1 — category click
  level1.addEventListener("click", e => {
    const btn = e.target.closest(".op-l1-btn");
    if (!btn) return;
    const cat = btn.dataset.cat;
    level1.querySelectorAll(".op-l1-btn").forEach(b => b.classList.toggle("active", b === btn));
    document.querySelectorAll(".op-level2").forEach(p => { p.hidden = true; });
    document.getElementById("op-level2-" + cat).hidden = false;
    const s = loadState(); s["_cat"] = cat; saveState(s);
  });

  // Level 2 — single-open sub-group accordion
  document.querySelectorAll(".op-group").forEach(group => {
    const grpId = group.dataset.grpId;
    // Default collapsed; open only if explicitly saved as open
    const wasOpen = grpId && state[grpId] === false;
    group.classList.toggle("collapsed", !wasOpen);

    group.querySelector(".op-group-label").addEventListener("click", () => {
      const collapsed = group.classList.contains("collapsed");
      if (collapsed) {
        // Collapse all siblings in the same panel (single-open per category)
        const panel = group.closest(".op-level2");
        panel.querySelectorAll(".op-group").forEach(g => {
          if (g !== group) {
            g.classList.add("collapsed");
            if (g.dataset.grpId) {
              const s = loadState(); s[g.dataset.grpId] = true; saveState(s);
            }
          }
        });
        group.classList.remove("collapsed");
      } else {
        group.classList.add("collapsed");
      }
      if (grpId) {
        const s = loadState(); s[grpId] = group.classList.contains("collapsed"); saveState(s);
      }
    });
  });
})();

// ── WASM SIMD128 detection ────────────────────────────────────────────────────
(function detectSimd() {
  // Minimal WASM module containing a v128.const instruction (SIMD128 proposal).
  // If the browser's WASM engine validates it, SIMD128 is supported and the VM
  // will map vector ops to native instructions (SSE/AVX on x86, NEON on ARM).
  const probe = new Uint8Array([
    0x00,0x61,0x73,0x6d, 0x01,0x00,0x00,0x00, // magic + version
    0x01,0x05,0x01,0x60, 0x00,0x01,0x7b,       // type section: () -> v128
    0x03,0x02,0x01,0x00,                        // function section: fn 0
    0x0a,0x16,0x01,0x14, 0x00,                  // code section: 1 fn, body=20B, 0 locals
    0xfd,0x0c,                                  // v128.const
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,  // 16-byte immediate
    0x0b                                        // end
  ]);
  let supported = false;
  try { supported = WebAssembly.validate(probe); } catch (_) {}
  const el = document.getElementById("simd-badge");
  if (supported) {
    el.textContent = "SIMD128 \u2713";
    el.classList.add("simd-supported");
    el.title = "WASM SIMD128 supported \u2014 browser maps vector ops to native instructions (SSE/AVX/NEON)";
  } else {
    el.textContent = "SIMD128 \u2717";
    el.classList.add("simd-unsupported");
    el.title = "WASM SIMD128 not supported \u2014 scalar fallback in use";
  }
})();

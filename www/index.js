import { M128i } from "visual-intrinsics";

// ── State ────────────────────────────────────────────────────────────────────
let regA = M128i.new();
let regB = M128i.new();
let regR = M128i.new();
let lastOp = "—";

const views = { a: "epi8", b: "epi8", r: "epi8" };

// ── Lane count per view ───────────────────────────────────────────────────────
const LANE_COUNTS = { epi8: 16, epi16: 8, epi32: 4, epi64: 2, bits: 1 };
const BITS_PER_LANE = { epi8: 8, epi16: 16, epi32: 32, epi64: 64, bits: 128 };

// ── Build the 32-wide bit grid (4 rows × 32 cols = 128 bits) ─────────────────
function buildGrid(id) {
  const container = document.getElementById(id);
  container.innerHTML = "";
  for (let i = 0; i < 128; i++) {
    const div = document.createElement("div");
    div.className = "bit off";
    div.dataset.idx = i; // bit index, 0 = MSB
    container.appendChild(div);
  }
}

// ── Render a register into its grid and lane table ────────────────────────────
function renderReg(key, reg) {
  const gridId  = "grid-" + key;
  const lanesId = "lanes-" + key;
  const view    = views[key];

  const bits = reg.get_bits(); // 128-char string, idx 0 = MSB
  const laneCount = LANE_COUNTS[view];
  const bitsPerLane = BITS_PER_LANE[view];

  // ── bit grid ─────────────────────────────────────────────────────────────
  const cells = document.getElementById(gridId).children;
  for (let i = 0; i < 128; i++) {
    const cell = cells[i];
    const on = bits[i] === "1";
    cell.className = "bit " + (on ? "on" : "off");

    if (view === "bits") {
      cell.removeAttribute("data-lane");
    } else {
      // which lane does bit index i belong to?  i=0 is MSB of MSB lane
      const laneFromMsb = Math.floor(i / bitsPerLane);
      cell.dataset.lane = laneFromMsb % 8;
    }
  }

  // ── lane table ────────────────────────────────────────────────────────────
  const table = document.getElementById(lanesId);
  table.innerHTML = "";

  let vals, hexBytes;
  if (view === "epi8")  vals = JSON.parse(reg.get_epi8());
  else if (view === "epi16") vals = JSON.parse(reg.get_epi16());
  else if (view === "epi32") vals = JSON.parse(reg.get_epi32());
  else if (view === "epi64") vals = JSON.parse(reg.get_epi64()).map(String);
  else { // bits
    vals = [reg.to_hex()];
  }

  // Values come from Rust index 0 = least-significant lane.
  // Display in MSB-first order to match the bit grid.
  const laneCountAct = vals.length;
  for (let li = laneCountAct - 1; li >= 0; li--) {
    const row = document.createElement("div");
    row.className = "lane-row";

    const idxSpan = document.createElement("span");
    idxSpan.className = "lane-idx";
    idxSpan.textContent = "[" + li + "]";

    const valSpan = document.createElement("span");
    valSpan.className = "lane-val";
    valSpan.textContent = vals[li];

    const hexSpan = document.createElement("span");
    hexSpan.className = "lane-hex";
    if (view !== "bits") {
      const nibblesPerLane = bitsPerLane / 4;
      let v = BigInt(vals[li]);
      if (v < 0n) v += (1n << BigInt(bitsPerLane));
      hexSpan.textContent = "0x" + v.toString(16).toUpperCase().padStart(nibblesPerLane, "0");
    }

    row.appendChild(idxSpan);
    row.appendChild(valSpan);
    row.appendChild(hexSpan);
    table.appendChild(row);
  }

  // update hex input if it's A or B
  if (key === "a") document.getElementById("hex-a").value = reg.to_hex();
  if (key === "b") document.getElementById("hex-b").value = reg.to_hex();
  if (key === "r") document.getElementById("result-hex").textContent = reg.to_hex();
}

// ── Render all three registers ────────────────────────────────────────────────
function renderAll() {
  renderReg("a", regA);
  renderReg("b", regB);
  renderReg("r", regR);
  document.getElementById("result-op-label").textContent = lastOp;
}

// ── View tab switching ────────────────────────────────────────────────────────
function setupTabs(tabsId, key) {
  const container = document.getElementById(tabsId);
  container.addEventListener("click", e => {
    if (!e.target.dataset.view) return;
    container.querySelectorAll("button").forEach(b => b.classList.remove("active"));
    e.target.classList.add("active");
    views[key] = e.target.dataset.view;
    renderReg(key, key === "a" ? regA : key === "b" ? regB : regR);
  });
}

// ── Hex set helpers ───────────────────────────────────────────────────────────
function trySetFromHex(inputId, key) {
  const raw = document.getElementById(inputId).value.trim();
  try {
    const reg = M128i.from_hex(raw || "0");
    if (key === "a") regA = reg;
    else             regB = reg;
    renderReg(key, key === "a" ? regA : regB);
  } catch (e) {
    alert("Invalid hex: " + e);
  }
}

// ── Operations ────────────────────────────────────────────────────────────────
function applyOp(op) {
  const bits = parseInt(document.getElementById("shift-bits").value) || 1;
  switch (op) {
    case "and":        regR = regA.and(regB);              lastOp = "A AND B"; break;
    case "or":         regR = regA.or(regB);               lastOp = "A OR B"; break;
    case "xor":        regR = regA.xor(regB);              lastOp = "A XOR B"; break;
    case "not-a":      regR = regA.not();                  lastOp = "NOT A"; break;
    case "not-b":      regR = regB.not();                  lastOp = "NOT B"; break;
    case "add_epi8":   regR = regA.add_epi8(regB);         lastOp = "ADD_EPI8(A,B)"; break;
    case "add_epi16":  regR = regA.add_epi16(regB);        lastOp = "ADD_EPI16(A,B)"; break;
    case "add_epi32":  regR = regA.add_epi32(regB);        lastOp = "ADD_EPI32(A,B)"; break;
    case "shl":        regR = regA.shift_left_bits(bits);  lastOp = `SHL A ${bits}`; break;
    case "shr":        regR = regA.shift_right_bits(bits); lastOp = `SHR A ${bits}`; break;
    case "a-to-b":     regB = regA.clone_reg();            lastOp = "A→B"; break;
    case "b-to-a":     regA = regB.clone_reg();            lastOp = "B→A"; break;
    case "result-to-a": regA = regR.clone_reg();           lastOp = "R→A"; break;
    case "result-to-b": regB = regR.clone_reg();           lastOp = "R→B"; break;
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

document.getElementById("btn-set-a").addEventListener("click", () => trySetFromHex("hex-a", "a"));
document.getElementById("btn-set-b").addEventListener("click", () => trySetFromHex("hex-b", "b"));
document.getElementById("hex-a").addEventListener("keydown", e => { if (e.key === "Enter") trySetFromHex("hex-a", "a"); });
document.getElementById("hex-b").addEventListener("keydown", e => { if (e.key === "Enter") trySetFromHex("hex-b", "b"); });

document.getElementById("btn-clear-a").addEventListener("click", () => { regA = M128i.new(); renderReg("a", regA); });
document.getElementById("btn-clear-b").addEventListener("click", () => { regB = M128i.new(); renderReg("b", regB); });

document.querySelectorAll(".op-btn[data-op]").forEach(btn => {
  btn.addEventListener("click", () => applyOp(btn.dataset.op));
});

// Seed registers with non-trivial demo values
regA = M128i.from_epi32(0x00ff00ff, 0x0f0f0f0f, 0xaaaaaaaa, 0x12345678);
regB = M128i.from_epi32(0xff00ff00, 0xf0f0f0f0, 0x55555555, 0x87654321);
renderAll();



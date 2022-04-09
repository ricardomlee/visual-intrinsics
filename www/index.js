import {M128i} from "visual-intrinsics";

const pre = document.getElementById("vi-canvas");
const m128i = M128i.new();
document.getElementById("myBtn").onclick = function() {myClick()};
document.getElementById("minusBtn").onclick = function() { minusClick() };

pre.addEventListener('mousedown', e => {
  pre.textContent = m128i.print_hex();
});

pre.addEventListener('mouseup', e => {
  pre.textContent = m128i.render();
});

function myClick() {
  m128i.add_one();
  pre.textContent = m128i.render();
}

function minusClick() {
  m128i.minus_one();
  pre.textContent = m128i.render();
}

pre.textContent = m128i.render();


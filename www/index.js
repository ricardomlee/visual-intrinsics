import {M128i} from "visual-intrinsics";

const pre = document.getElementById("vi-canvas");
const m128i = M128i.new();
document.getElementById("myBtn").onclick = function() {myClick()};

function myClick() {
  pre.textContent = "";
  m128i.add_one();
  pre.textContent = m128i.render();
}

pre.textContent = m128i.render();


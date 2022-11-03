import init, { WebState } from "./pkg/mandelbrot.js";

await init();

const state = await WebState.new(window.innerWidth, window.innerHeight);
state.draw();

window.addEventListener("resize", () => {
  state.resize(window.innerWidth, window.innerHeight);
  state.draw();
});

const canvas = document.querySelector("canvas");

canvas.addEventListener("wheel", (e) => {
  const { deltaY, x, y } = e;

  state.zoom(deltaY * -0.01);

  const [nx, ny] = get_normal_direction(
    x,
    y,
    window.innerWidth,
    window.innerHeight
  );

  let mul = deltaY;
  if (deltaY < 0) {
    mul *= 2;
  }

  state.pan(nx * mul, ny * mul);
  state.draw();
});

window.addEventListener("keypress", (e) => {
  const { key } = e;
  let prev = false;

  switch (key) {
    case "r": {
      state.reset();
      state.draw();
      prev = true;
      break;
    }
  }

  if (prev) {
    e.preventDefault();
  }
});

window.addEventListener("mousedown", () => {
  lastX = undefined;
  lastY = undefined;
  window.addEventListener("mousemove", handle_mouse_move);
});

window.addEventListener("mouseup", () => {
  window.removeEventListener("mousemove", handle_mouse_move);
});

// TODO: this could be done better than with global vars
let lastX;
let lastY;
function handle_mouse_move(e) {
  let { x, y } = e;
  if (lastX == undefined) lastX = x;
  if (lastY == undefined) lastY = y;
  state.pan(x - lastX, y - lastY);
  state.draw();

  lastX = x;
  lastY = y;
}

function get_normal_direction(x, y, ox, oy) {
  let nx = 0;
  let ny = 0;

  nx = x - ox / 2;
  ny = y - oy / 2;
  // FIXME: too slow
  const d = Math.sqrt(nx ** 2 + ny ** 2);

  nx /= d;
  ny /= d;

  return [nx, ny];
}

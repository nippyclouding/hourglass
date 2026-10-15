"use strict";

const dismiss = document.querySelector("#dismiss");
const message = document.querySelector("#message");
const invoke = window.__TAURI__?.core.invoke;
let pending = false;

async function acknowledge() {
  if (!invoke || pending) return;
  pending = true;
  dismiss.disabled = true;
  message.textContent = "";
  try {
    await invoke("dismiss_completion");
  } catch (error) {
    message.textContent = String(error);
    pending = false;
    dismiss.disabled = false;
    dismiss.focus();
  }
}

dismiss.addEventListener("click", acknowledge);
document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" && event.key !== "Escape") return;
  event.preventDefault();
  return acknowledge();
});

if (invoke) {
  dismiss.focus();
} else {
  dismiss.disabled = true;
  message.textContent = "Open Hourglass from the menu bar or system tray.";
}

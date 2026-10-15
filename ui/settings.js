"use strict";

const form = document.querySelector("#settings-form");
const minutes = document.querySelector("#minutes");
const startup = document.querySelector("#startup");
const save = document.querySelector("#save");
const quit = document.querySelector("#quit");
const restart = document.querySelector("#restart");
const message = document.querySelector("#message");
const invoke = window.__TAURI__?.core.invoke;
let ready = false;
let saving = false;
let canRestart = false;

function showError(error) {
  message.textContent = String(error);
  message.classList.add("error");
}

function setDisabled(disabled) {
  minutes.disabled = disabled;
  startup.disabled = disabled;
  save.disabled = disabled;
  restart.disabled = disabled || !canRestart;
}

async function load() {
  if (!invoke) {
    showError("Open Hourglass from the menu bar or system tray.");
    quit.disabled = true;
    return;
  }
  try {
    const settings = await invoke("get_settings");
    minutes.value = settings.minutes;
    startup.checked = settings.launchAtStartup;
    canRestart = Boolean(settings.canRestart);
    ready = true;
    setDisabled(false);
    minutes.focus();
    minutes.select();
  } catch (error) {
    showError(error);
  }
}

function readMinutes() {
  if (!form.reportValidity()) return null;
  const value = Number(minutes.value);
  if (minutes.value.trim() === "" || !Number.isInteger(value) || value < 1 || value > 525600) {
    showError("Enter a whole number from 1 to 525600 minutes.");
    return null;
  }
  return value;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!ready || saving) return;
  const value = readMinutes();
  if (value === null) return;
  saving = true;
  setDisabled(true);
  message.textContent = "";
  message.classList.remove("error");
  try {
    await invoke("save_settings", { minutes: value, launchAtStartup: startup.checked });
    await invoke("close_settings");
  } catch (error) {
    showError(error);
  } finally {
    saving = false;
    setDisabled(false);
  }
});

quit.addEventListener("click", async () => {
  if (!invoke || saving) return;
  try { await invoke("quit"); } catch (error) { showError(error); }
});

restart.addEventListener("click", async () => {
  if (!ready || saving || !canRestart) return;
  const value = readMinutes();
  if (value === null) return;
  saving = true;
  setDisabled(true);
  message.textContent = "";
  message.classList.remove("error");
  try {
    await invoke("restart_timer", { minutes: value });
    await invoke("close_settings");
  } catch (error) {
    showError(error);
  } finally {
    saving = false;
    setDisabled(false);
  }
});

document.addEventListener("keydown", async (event) => {
  if (event.key !== "Escape" || !invoke || saving) return;
  try { await invoke("close_settings"); } catch (error) { showError(error); }
});

load();

const test = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const source = readFileSync(path.join(__dirname, "../ui/settings.js"), "utf8");

function setup(handler = async () => ({ minutes: 50, launchAtStartup: false }), native = true) {
  const calls = [];
  const elements = {};
  for (const name of ["settings-form", "minutes", "startup", "save", "quit", "restart", "message"]) {
    elements[name] = {
      value: "50", checked: false, disabled: true, textContent: "", events: {},
      classList: { add() {}, remove() {} },
      addEventListener(name, callback) { this.events[name] = callback; },
      reportValidity() { return true; }, focus() {}, select() {},
    };
  }
  const events = {};
  const invoke = async (command, args) => {
    calls.push({ command, args });
    return handler(command, args);
  };
  vm.runInNewContext(source, {
    window: native ? { __TAURI__: { core: { invoke } } } : {},
    document: {
      querySelector: (selector) => elements[selector.slice(1)],
      addEventListener: (name, callback) => { events[name] = callback; },
    },
  });
  return {
    elements, calls, events,
    async loaded() { await new Promise(setImmediate); },
    async submit(value) {
      elements.minutes.value = value;
      await elements["settings-form"].events.submit({ preventDefault() {} });
    },
  };
}

test("loads persisted settings before allowing edits", async () => {
  const app = setup(async () => ({ minutes: 90, launchAtStartup: true }));
  assert.equal(app.elements.save.disabled, true);
  await app.loaded();
  assert.equal(app.elements.minutes.value, 90);
  assert.equal(app.elements.startup.checked, true);
  assert.equal(app.elements.save.disabled, false);
});

test("invalid input never invokes save", async () => {
  const app = setup();
  await app.loaded();
  for (const value of ["", " ", "0", "-1", "1.5", "abc", "525601"]) await app.submit(value);
  assert.equal(app.calls.filter(call => call.command === "save_settings").length, 0);
});

test("saves both settings and closes only after persistence succeeds", async () => {
  const app = setup();
  await app.loaded();
  app.elements.startup.checked = true;
  await app.submit("120");
  assert.deepEqual(app.calls.map(call => call.command), ["get_settings", "save_settings", "close_settings"]);
  assert.equal(app.calls[1].args.minutes, 120);
  assert.equal(app.calls[1].args.launchAtStartup, true);
});

test("a failed save stays open and re-enables editing", async () => {
  const app = setup(async command => {
    if (command === "save_settings") throw new Error("Disk is full");
    return { minutes: 50, launchAtStartup: false };
  });
  await app.loaded();
  await app.submit("25");
  assert.match(app.elements.message.textContent, /Disk is full/);
  assert.equal(app.elements.save.disabled, false);
  assert.equal(app.calls.some(call => call.command === "close_settings"), false);
});

test("duplicate submissions cannot race", async () => {
  let finish;
  const app = setup(command => command === "save_settings"
    ? new Promise(resolve => { finish = resolve; })
    : Promise.resolve({ minutes: 50, launchAtStartup: false }));
  await app.loaded();
  const first = app.submit("25");
  await app.submit("90");
  assert.equal(app.calls.filter(call => call.command === "save_settings").length, 1);
  finish();
  await first;
});

test("a settings load failure cannot overwrite stored data with defaults", async () => {
  const app = setup(async () => { throw new Error("Cannot read settings"); });
  await app.loaded();
  await app.submit("25");
  assert.equal(app.elements.save.disabled, true);
  assert.match(app.elements.message.textContent, /Cannot read settings/);
  assert.equal(app.calls.length, 1);
});

test("Escape closes the settings window and Quit exits the app", async () => {
  const app = setup();
  await app.loaded();
  await app.events.keydown({ key: "Escape" });
  await app.elements.quit.events.click();
  assert.deepEqual(app.calls.map(call => call.command), ["get_settings", "close_settings", "quit"]);
});

test("browser preview does not offer nonfunctional actions", async () => {
  const app = setup(undefined, false);
  await app.loaded();
  assert.equal(app.elements.save.disabled, true);
  assert.equal(app.elements.quit.disabled, true);
  assert.equal(app.elements.restart.disabled, true);
  assert.equal(app.calls.length, 0);
});

test("restart is disabled without an active timer", async () => {
  const app = setup();
  await app.loaded();
  assert.equal(app.elements.restart.disabled, true);
  await app.elements.restart.events.click();
  assert.equal(app.calls.length, 1);
});

test("restart does not save or use unsaved time and startup edits", async () => {
  const app = setup(async () => ({ minutes: 50, launchAtStartup: false, canRestart: true }));
  await app.loaded();
  assert.equal(app.elements.restart.disabled, false);
  app.elements.minutes.value = "25";
  app.elements.startup.checked = true;
  await app.elements.restart.events.click();
  assert.deepEqual(app.calls.map(call => call.command), ["get_settings", "restart_timer", "close_settings"]);
  assert.equal(app.calls[1].args, undefined);
});

test("a failed restart stays open with an error", async () => {
  const app = setup(async command => {
    if (command === "restart_timer") throw new Error("Timer already finished");
    return { minutes: 50, launchAtStartup: false, canRestart: true };
  });
  await app.loaded();
  await app.elements.restart.events.click();
  assert.match(app.elements.message.textContent, /Timer already finished/);
  assert.equal(app.elements.restart.disabled, false);
  assert.equal(app.calls.some(call => call.command === "close_settings"), false);
});

test("restart blocks duplicate restart, save, and quit commands while pending", async () => {
  let finish;
  const app = setup(command => command === "restart_timer"
    ? new Promise(resolve => { finish = resolve; })
    : Promise.resolve({ minutes: 50, launchAtStartup: false, canRestart: true }));
  await app.loaded();
  const first = app.elements.restart.events.click();
  await app.elements.restart.events.click();
  await app.submit("90");
  await app.elements.quit.events.click();
  assert.deepEqual(app.calls.map(call => call.command), ["get_settings", "restart_timer"]);
  finish();
  await first;
});

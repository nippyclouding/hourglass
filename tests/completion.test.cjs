const test = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const source = readFileSync(path.join(__dirname, "../ui/completion.js"), "utf8");

function setup(handler = async () => {}, native = true) {
  const calls = [];
  const events = {};
  const elements = {
    dismiss: {
      disabled: false,
      focused: false,
      events: {},
      focus() { this.focused = true; },
      addEventListener(name, callback) { this.events[name] = callback; },
    },
    message: { textContent: "" },
  };
  const invoke = async (command) => {
    calls.push(command);
    return handler(command);
  };
  vm.runInNewContext(source, {
    window: native ? { __TAURI__: { core: { invoke } } } : {},
    document: {
      querySelector: selector => elements[selector.slice(1)],
      addEventListener: (name, callback) => { events[name] = callback; },
    },
  });
  return {
    calls, elements, events,
    click: () => elements.dismiss.events.click(),
    async key(key) {
      let prevented = false;
      await events.keydown({ key, preventDefault() { prevented = true; } });
      return prevented;
    },
  };
}

test("OK focuses by default and acknowledges completion through the native command", async () => {
  const app = setup();
  assert.equal(app.elements.dismiss.focused, true);
  await app.click();
  assert.deepEqual(app.calls, ["dismiss_completion"]);
  assert.equal(app.elements.dismiss.disabled, true);
});

test("Enter and Escape acknowledge the alert and prevent duplicate native key actions", async () => {
  for (const key of ["Enter", "Escape"]) {
    const app = setup();
    assert.equal(await app.key("a"), false);
    assert.equal(app.calls.length, 0);
    assert.equal(await app.key(key), true);
    assert.deepEqual(app.calls, ["dismiss_completion"]);
  }
});

test("repeated clicks and key presses cannot race a pending dismissal", async () => {
  let finish;
  const app = setup(() => new Promise(resolve => { finish = resolve; }));
  const first = app.click();
  await app.click();
  await app.key("Enter");
  await app.key("Escape");
  assert.deepEqual(app.calls, ["dismiss_completion"]);
  finish();
  await first;
});

test("a failed dismissal keeps the alert usable and permits a retry", async () => {
  let attempts = 0;
  const app = setup(async () => {
    if (++attempts === 1) throw new Error("Could not close the alert");
  });
  await app.click();
  assert.match(app.elements.message.textContent, /Could not close the alert/);
  assert.equal(app.elements.dismiss.disabled, false);
  await app.key("Enter");
  assert.deepEqual(app.calls, ["dismiss_completion", "dismiss_completion"]);
  assert.equal(app.elements.message.textContent, "");
  assert.equal(app.elements.dismiss.disabled, true);
});

test("a browser preview explains the missing native API without invoking commands", async () => {
  const app = setup(undefined, false);
  assert.equal(app.elements.dismiss.disabled, true);
  assert.match(app.elements.message.textContent, /Open Hourglass/);
  await app.click();
  await app.key("Enter");
  await app.key("Escape");
  assert.deepEqual(app.calls, []);
});

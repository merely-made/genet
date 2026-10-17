// SPDX-License-Identifier: MPL-2.0
// Serve input_defaults.html beside freshly built ortet.js/ortet_bg.wasm.
// PLAYWRIGHT_MODULE=/path/to/playwright CHROMIUM_PATH=/path/to/chrome \
//   node input_defaults.cjs http://127.0.0.1:18744/input_defaults.html receipt.json
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const assert = require('node:assert/strict');
(async () => {
  const browser = await chromium.launch({executablePath: process.env.CHROMIUM_PATH || undefined,
    headless: true, args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan', '--use-angle=vulkan', '--no-sandbox']});
  const receipt = { observations: [], errors: [] };
  try {
    const page = await browser.newPage({viewport: {width: 800, height: 650}});
    page.on('pageerror', e => receipt.errors.push(String(e)));
    await page.goto(process.argv[2]);
    await page.waitForFunction(() => document.querySelector('#receipt')?.dataset.status !== 'pending', {}, {timeout: 45000});
    assert.equal(await page.locator('#receipt').getAttribute('data-status'), 'ready', await page.locator('#receipt').textContent());
    receipt.adapter = await page.evaluate(async () => {
      const adapter = await navigator.gpu.requestAdapter();
      return adapter ? {vendor: adapter.info.vendor, architecture: adapter.info.architecture, device: adapter.info.device, description: adapter.info.description, isFallbackAdapter: adapter.info.isFallbackAdapter} : null;
    });
    const capture = () => page.evaluate(() => window.ortetInputFixture.capture());
    const click = async name => {
      const target = await page.evaluate(name => window.ortetInputFixture.targets[name], name);
      await page.locator('#ortet').click({position: target});
    };
    const press = async (key, scrolls = false) => {
      const before = await capture();
      await page.keyboard.press(key);
      await page.waitForTimeout(200);
      let after = await capture();
      const deadline = Date.now() + 5000;
      while (scrolls && after.stripHash === before.stripHash && Date.now() < deadline) after = await capture();
      const event = after.events.at(-1);
      assert.equal(after.events.length, before.events.length + 1, 'canvas received keydown');
      assert.equal(event.isTrusted, true, 'actual browser key');
      assert.equal(event.key, key === 'Space' ? ' ' : key);
      assert.equal(await page.evaluate(() => document.activeElement.id), 'ortet');
      receipt.observations.push({key, before, after});
      return {before, after, event};
    };
    for (const target of ['input', 'textarea']) {
      await click(target);
      for (const key of ['ArrowDown', 'Home', 'End', 'Space']) {
        const {before, after, event} = await press(key);
        assert.equal(event.defaultPrevented, true, `${target} ${key} prevents browser default`);
        assert.equal(after.outerScrollY, before.outerScrollY, `${target} ${key} outer scroll`);
        assert.equal(after.stripHash, before.stripHash, `${target} ${key} product scroll`);
      }
      await press('Home');
      const beforeEdit = await capture();
      await page.keyboard.type('WWWW');
      let afterEdit = await capture();
      const editDeadline = Date.now() + 5000;
      while (afterEdit.controlInk[target] <= beforeEdit.controlInk[target] + 25 && Date.now() < editDeadline) afterEdit = await capture();
      receipt.observations.push({target, typed: 'WWWW', before: beforeEdit, after: afterEdit});
      await page.screenshot({path: process.argv[3] + '.png'});
      assert.ok(afterEdit.controlInk[target] > beforeEdit.controlInk[target] + 25, `${target} displays typed glyphs, beyond caret ink`);
      assert.equal(afterEdit.stripHash, beforeEdit.stripHash, 'typing does not scroll product');
      assert.equal(afterEdit.outerScrollY, beforeEdit.outerScrollY, 'typing does not scroll browser');
    }
    await click('noneditable');
    const ignored = await press('F8');
    assert.equal(ignored.event.defaultPrevented, false, 'ignored key remains available');
    const down = await press('ArrowDown', true);
    assert.equal(down.event.defaultPrevented, true, 'product scroll consumes arrow');
    assert.notEqual(down.after.stripHash, down.before.stripHash, 'product pixels move');
    assert.equal(down.after.outerScrollY, down.before.outerScrollY, 'host does not double-scroll');
    const space = await press('Space', true);
    assert.equal(space.event.defaultPrevented, true, 'product consumes Space');
    assert.notEqual(space.after.stripHash, space.before.stripHash, 'Space moves product pixels');
    assert.equal(space.after.outerScrollY, space.before.outerScrollY, 'Space does not scroll host');
    const end = await press('End', true);
    assert.equal(end.event.defaultPrevented, true, 'product consumes End');
    assert.notEqual(end.after.stripHash, end.before.stripHash, 'End moves product pixels');
    assert.equal(end.after.outerScrollY, end.before.outerScrollY, 'End does not scroll host');
    const boundary = await press('End');
    assert.equal(boundary.event.defaultPrevented, false, 'exhausted product scroll allows browser default');
    assert.ok(boundary.after.outerScrollY > boundary.before.outerScrollY, 'outer browser scroll occurs at boundary');
    assert.equal(receipt.errors.length, 0, 'no page errors');
    receipt.status = 'passed';
  } catch (error) {
    receipt.status = 'failed'; receipt.failure = String(error); process.exitCode = 1;
  } finally {
    fs.writeFileSync(process.argv[3], JSON.stringify(receipt, null, 2) + '\n');
    console.log(JSON.stringify({status: receipt.status, failure: receipt.failure, actions: receipt.observations.length}));
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });

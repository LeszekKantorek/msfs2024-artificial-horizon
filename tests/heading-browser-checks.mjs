import assert from 'node:assert/strict';

export async function checkHeading(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', hold: false, attitude_only: false, heading_deg: 359, selected_heading_deg: 1 });
    await page.goto(url + path);
    await page.locator('#heading[data-state="available"]').waitFor();
    await page.evaluate(() => { window.headingNodes = [...document.querySelectorAll('#heading *')]; });
    for (const [width, height] of [[320, 240], [326, 246], [320, 480], [568, 240], [844, 320]]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      for (const [heading, selected, text] of [[359, 1, '359°'], [0, 0, '360°'], [1, 359, '001°'],
        [354, 354, '354°'], [5, 5, '005°'], [359.5, 0, '360°'], [354, 90, '354°'], [354, 180, '354°']]) {
        setFixture({ heading_deg: heading, selected_heading_deg: selected });
        await page.waitForFunction(({ heading, selected }) => document.querySelector('#heading').getAttribute('aria-label') ===
          `Magnetic heading ${heading} degrees, selected heading ${selected} degrees`, { heading, selected });
        const paint = await page.evaluate(() => {
          const group = document.querySelector('#heading'), window = group.querySelector('.heading-window');
          const value = document.querySelector('#heading-value'), bug = document.querySelector('#selected-heading-bug');
          const box = window.getBoundingClientRect(), text = value.getBoundingClientRect();
          const bounds = bug.getBoundingClientRect(), clip = document.querySelector('#heading-tape-clip rect').getBBox();
          const origin = new DOMPoint(0, 0).matrixTransform(group.getScreenCTM());
          const point = (x, y) => new DOMPoint(x, y).matrixTransform(bug.getScreenCTM());
          const cyan = point(7, 28), notch = point(0, 28);
          const labels = [...group.querySelectorAll('.heading-label')].filter(n => getComputedStyle(n).visibility !== 'hidden' && getComputedStyle(n).display !== 'none');
          return { width: clip.width, center: bug.transform.baseVal.consolidate().matrix.e, text: value.textContent,
            textContained: text.left > box.left && text.right < box.right && text.top > box.top && text.bottom < origin.y + 22,
            bugContained: bounds.left - 0.5 >= origin.x - 0.0001 && bounds.right + 0.5 <= origin.x + clip.width + 0.0001 && bounds.bottom + 0.5 <= origin.y + clip.height,
            labelsContained: labels.every(n => { const b = n.getBoundingClientRect(); return b.left >= origin.x && b.right <= origin.x + clip.width && (b.right < box.left || b.left > box.right); }),
            cyanPainted: document.elementFromPoint(cyan.x, cyan.y) === bug,
            notchCovered: document.elementFromPoint(notch.x, notch.y) === window,
            valueSize: getComputedStyle(value).fontSize, labelSize: getComputedStyle(group.querySelector('.heading-label')).fontSize,
            color: getComputedStyle(bug).fill, stable: globalThis.headingNodes.every(n => n.isConnected) };
        });
        const difference = (selected - heading + 540) % 360 - 180;
        const expected = Math.min(paint.width - 8.5, Math.max(8.5, paint.width / 2 + difference * 4));
        assert.ok(Math.abs(paint.center - expected) < 1e-4, JSON.stringify({ heading, selected, paint }));
        assert.equal(paint.text, text);
        assert.ok(paint.textContained && paint.bugContained && paint.labelsContained && paint.stable,
          JSON.stringify({ width, height, heading, selected, paint }));
        assert.equal(paint.valueSize, '14px'); assert.equal(paint.labelSize, '11px');
        assert.equal(paint.color, 'rgb(134, 222, 245)');
        if (heading === selected) assert.ok(paint.cyanPainted && paint.notchCovered, JSON.stringify(paint));
        if ((width === 320 || width === 568) && [0, 359, 354].includes(heading)) {
          await page.screenshot({ path: `.local/pfd-browser/heading-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${heading}-${selected}.png` });
        }
      }
      for (const [heading, selected, state, selectedAvailable] of [[0, null, 'available', 'false'], [0, 360, 'available', 'false'],
        [null, 1, 'unavailable', 'false'], [360, 1, 'unavailable', 'false'], ['bad', 1, 'unavailable', 'false']]) {
        setFixture({ heading_deg: heading, selected_heading_deg: selected });
        await page.waitForFunction(({ state, selectedAvailable }) => document.querySelector('#heading').dataset.state === state &&
          document.querySelector('#heading').dataset.selectedAvailable === selectedAvailable, { state, selectedAvailable });
        assert.equal(await page.locator('#selected-heading-bug').evaluate(n => getComputedStyle(n).display), 'none');
        assert.equal(await page.locator('#selected-heading-bug').getAttribute('transform'), null);
        assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
        if (state === 'unavailable') {
          assert.equal(await page.locator('#heading-value').textContent(), 'X');
          assert.equal(await page.locator('#heading-value').evaluate(n => getComputedStyle(n).fill), 'rgb(255, 150, 144)');
          assert.equal(await page.locator('#heading-scale').evaluate(n => getComputedStyle(n).display), 'none');
        }
      }
    }
    setFixture({ heading_deg: 359, selected_heading_deg: 1 });
    await page.locator('#heading[data-state="available"]').waitFor();
    setFixture({ hold: true });
    await page.locator('.instrument[data-available="false"]').waitFor();
    assert.equal(await page.locator('#heading-value').textContent(), 'X');
    assert.equal(await page.locator('#selected-heading-bug').getAttribute('transform'), null);
    setFixture({ hold: false, state: 'paused' });
    await page.waitForFunction(() => document.querySelector('#telemetry-status').textContent.includes('paused'));
    assert.equal(await page.locator('#heading-value').textContent(), 'X');
    setFixture({ state: 'live', heading_deg: 354, selected_heading_deg: 354 });
    await page.locator('#heading[data-state="available"]').waitFor();
    assert.equal(await page.locator('#heading-value').textContent(), '354°');
    assert.equal(await page.evaluate(() => globalThis.headingNodes.every(n => n.isConnected)), true);
  }
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

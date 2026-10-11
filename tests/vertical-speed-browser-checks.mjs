import assert from 'node:assert/strict';

export async function checkVerticalSpeed(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', attitude_only: false, altitude_ft: 10060, selected_altitude_ft: 10000,
      baro_inhg: 29.92, vertical_speed_fpm: 0, hold: false, pitch_deg: 0 });
    await page.goto(url + path);
    await page.locator('#vertical-speed[data-state="available"]').waitFor();
    await page.evaluate(() => { window.vsiNodes = [...document.querySelectorAll('#vertical-speed, #vertical-speed *')]; });
    for (const [width, height] of [[320, 240], [320, 480], [568, 240], [326, 246]]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      for (const [fpm, text, state] of [[0, '0', 'available'], [-0.1, '0', 'available'],
        [1500, '+1500', 'available'], [-1500, '-1500', 'available'],
        [2000.01, '+2000', 'available'], [-2000.01, '-2000', 'available'],
        [4881.36, '+4881', 'available'], [9999, '+9999', 'available'], [-9999, '-9999', 'available'],
        [9999.01, '+OVR', 'overflow'], [-9999.01, '-OVR', 'overflow'],
        [Number.MAX_VALUE, '+OVR', 'overflow'], [-Number.MAX_VALUE, '-OVR', 'overflow']]) {
        setFixture({ vertical_speed_fpm: fpm });
        await page.waitForFunction(({ text, fpm }) => document.querySelector('#vertical-speed-value').textContent === text &&
          document.querySelector('#vertical-speed').getAttribute('aria-label') === `Vertical speed ${fpm} feet per minute`, { text, fpm });
        const painted = await page.evaluate(() => {
          const group = document.querySelector('#vertical-speed');
          const value = document.querySelector('#vertical-speed-value');
          const boxNode = document.querySelector('.vertical-speed-window');
          const altitudeNode = document.querySelector('.altitude-window');
          const box = boxNode.getBoundingClientRect(), altitude = altitudeNode.getBoundingClientRect();
          const valueBox = value.getBoundingClientRect();
          const altitudeDigits = document.querySelector('#altitude-digits > g:last-child > text');
          const textRightDifference = valueBox.right - altitudeDigits.getBoundingClientRect().right;
          const view = document.querySelector('svg').getBoundingClientRect();
          const boxStyle = getComputedStyle(boxNode), altitudeStyle = getComputedStyle(altitudeNode);
          const textStyle = getComputedStyle(value), digitStyle = getComputedStyle(document.querySelector('.altitude-digit'));
          const bug = document.querySelector('#selected-altitude-bug');
          const bugPoint = new DOMPoint(2, 0).matrixTransform(bug.getScreenCTM());
          const labelsClear = [...document.querySelectorAll('.altitude-label')].every(label => {
            if (!label.textContent || getComputedStyle(label).visibility === 'hidden') return true;
            const bounds = label.getBoundingClientRect();
            return bounds.bottom < box.top - 1 || bounds.top > box.bottom + 1;
          });
          const ticksClear = [...document.querySelectorAll('.altitude-tick')].every(tick => {
            const bounds = tick.getBoundingClientRect();
            return bounds.bottom < box.top || bounds.top > box.bottom || bounds.right + 0.5 < box.left - 1;
          });
          return { state: group.dataset.state, nested: group.parentElement.id === 'altitude',
            textContained: valueBox.left > box.left + 1 && valueBox.right < box.right - 1 && valueBox.top > box.top + 1 && valueBox.bottom < box.bottom - 1,
            windowContained: box.left >= view.left && box.right <= view.right && box.top >= view.top && box.bottom <= view.bottom,
            width: box.width, height: box.height, rightDifference: box.right - altitude.right, textRightDifference,
            paintedGap: box.top - 0.5 - (altitude.bottom + 0.5),
            sameWindow: ['fill', 'stroke', 'strokeWidth'].every(key => boxStyle[key] === altitudeStyle[key]),
            sameText: ['fontFamily', 'fontWeight', 'fill'].every(key => textStyle[key] === digitStyle[key]),
            fontSize: parseFloat(textStyle.fontSize), fontSizeDifference: parseFloat(digitStyle.fontSize) - parseFloat(textStyle.fontSize),
            bugPainted: document.elementFromPoint(bugPoint.x, bugPoint.y) === bug,
            labelsClear, ticksClear, stable: window.vsiNodes.every(node => node.isConnected),
            obsolete: document.querySelectorAll('#vertical-speed-scale, #vertical-speed-pointer, #vertical-speed-clip, #vertical-speed-off-scale, .vertical-speed-unit').length };
        });
        assert.equal(painted.state, state);
        assert.ok(Math.abs(painted.textRightDifference) < 0.02, `Text right edges: ${JSON.stringify({ width, height, fpm, painted })}`);
        assert.ok(painted.nested && painted.textContained && painted.windowContained && painted.sameWindow && painted.sameText &&
          painted.bugPainted && painted.labelsClear && painted.ticksClear && painted.stable, JSON.stringify({ width, height, fpm, painted }));
        assert.ok(Math.abs(painted.rightDifference) < 1e-4 && Math.abs(painted.paintedGap - 3) < 1e-4);
        assert.deepEqual([painted.width, painted.height, painted.obsolete, painted.fontSize, painted.fontSizeDifference], [51, 27, 0, 13, 1]);
      }
      setFixture({ vertical_speed_fpm: -9999, altitude_ft: -60, selected_altitude_ft: -100 });
      await page.waitForFunction(() => document.querySelector('#vertical-speed-value').textContent === '-9999' &&
        document.querySelector('#altitude-sign').textContent === '−');
      const bugVisible = await page.evaluate(() => {
        const bug = document.querySelector('#selected-altitude-bug');
        const point = new DOMPoint(2, 0).matrixTransform(bug.getScreenCTM());
        return document.elementFromPoint(point.x, point.y) === bug;
      });
      assert.equal(bugVisible, true);
      await page.screenshot({ path: `.local/pfd-browser/vsi-${path === '/' ? 'production' : 'layout'}-${width}x${height}-negative-capture.png` });
      setFixture({ vertical_speed_fpm: 1500, altitude_ft: null, baro_inhg: null });
      await page.locator('#altitude[data-state="unavailable"]').waitFor();
      await page.waitForFunction(() => document.querySelector('#vertical-speed-value').textContent === '+1500');
      assert.equal(await page.locator('#vertical-speed-value').evaluate(node => getComputedStyle(node).visibility), 'visible');
      setFixture({ altitude_ft: 100000, baro_inhg: 100 });
      await page.locator('#altitude[data-state="overflow"]').waitFor();
      assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'available');
      for (const invalid of [null, 'bad', false]) {
        setFixture({ vertical_speed_fpm: invalid, altitude_ft: 10060, selected_altitude_ft: 10000, baro_inhg: 29.92 });
        await page.locator('#vertical-speed[data-state="unavailable"]').waitFor();
        assert.equal(await page.locator('#vertical-speed-value').textContent(), 'X');
        assert.equal(await page.locator('#vertical-speed-value').evaluate(node => getComputedStyle(node).fill), 'rgb(255, 150, 144)');
        const invalidText = await page.evaluate(() => {
          const value = document.querySelector('#vertical-speed-value').getBoundingClientRect();
          const box = document.querySelector('.vertical-speed-window').getBoundingClientRect();
          const digits = document.querySelector('#altitude-digits > g:last-child > text').getBoundingClientRect();
          return { rightDifference: value.right - digits.right, contained: value.left > box.left + 1 && value.right < box.right - 1 };
        });
        assert.ok(invalidText.contained && Math.abs(invalidText.rightDifference) < 0.02, JSON.stringify(invalidText));
        assert.equal(await page.locator('.instrument').getAttribute('data-available'), 'true');
      }
      setFixture({ vertical_speed_fpm: -1500 });
      await page.locator('#vertical-speed[data-state="available"]').waitFor();
      await page.screenshot({ path: `.local/pfd-browser/vsi-${path === '/' ? 'production' : 'layout'}-${width}x${height}-descent.png` });
      setFixture({ state: 'paused' });
      await page.locator('.instrument[data-available="false"]').waitFor();
      assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'unavailable');
      assert.equal(await page.locator('#vertical-speed-value').textContent(), 'X');
      setFixture({ state: 'live', vertical_speed_fpm: 0 });
      await page.locator('.instrument[data-available="true"]').waitFor();
      setFixture({ hold: true });
      await page.locator('.instrument[data-available="false"]').waitFor();
      assert.equal(await page.locator('#vertical-speed').getAttribute('data-state'), 'unavailable');
      setFixture({ hold: false, vertical_speed_fpm: 0 });
      await page.locator('.instrument[data-available="true"]').waitFor();
    }
  }
  setFixture({ vertical_speed_fpm: 0 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

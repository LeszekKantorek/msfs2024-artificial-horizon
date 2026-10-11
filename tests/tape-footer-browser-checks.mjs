import assert from 'node:assert/strict';

export async function checkTapeFooters(page, setFixture, url) {
  for (const path of ['/', '/layout.html']) {
    setFixture({ state: 'live', hold: false, attitude_only: false, altitude_ft: 1000,
      ias_kt: 120, gs_kt: 157, baro_text: '23.83 inHg', pitch_deg: 0, roll_deg: 0 });
    await page.goto(url + path);
    await page.locator('#barometric-setting[data-state="available"]').waitFor();
    for (const [width, height] of [[320, 480], [326, 246], [568, 240], [844, 320]]) {
      await page.setViewportSize({ width, height });
      // Fractional bounds expose seams hidden by whole-pixel layouts.
      for (const padding of ['3px', '3.25px', '15px 24px 12px 12px']) {
        await page.locator('main').evaluate((n, padding) => { n.style.padding = padding; }, padding);
        await page.waitForFunction(() => {
          const svg = document.querySelector('svg'), b = svg.getBoundingClientRect();
          const [, , w, h] = svg.getAttribute('viewBox').split(' ').map(Number);
          return Math.abs(w - b.width) < 0.01 && Math.abs(h - b.height) < 0.01;
        });
        const geometry = await page.evaluate(() => {
          const bounds = selector => document.querySelector(selector).getBoundingClientRect();
          const style = selector => {
            const s = getComputedStyle(document.querySelector(selector));
            return [s.fontFamily, s.fontSize, s.fontWeight, s.fill, s.dominantBaseline];
          };
          const panel = bounds('svg'), ias = bounds('#airspeed > .instrument-background');
          const altitude = bounds('#altitude > .instrument-background');
          const gs = bounds('#ground-speed > .instrument-background');
          const baro = bounds('.barometric-background');
          const kt = bounds('#airspeed > .speed-unit'), ft = bounds('.altitude-unit');
          const altitudeWidth = document.querySelector('#altitude-tape-clip rect').getBBox().width;
          const edgesCovered = [panel.left + 0.1, panel.right - 0.1].every(x =>
            [ias.top + 0.1, gs.top - 0.1, panel.bottom - 0.1].every(y =>
              document.elementsFromPoint(x, y).some(n => n.matches('.instrument-background, .barometric-background'))));
          return {
            unitTexts: [...document.querySelectorAll('.speed-unit, .altitude-unit')].map(n => n.textContent),
            ktStyle: style('.speed-unit'), ftStyle: style('.altitude-unit'),
            labelDelta: (kt.top + kt.bottom - ft.top - ft.bottom) / 2,
            ktCenterDelta: (kt.left + kt.right - ias.left - ias.right) / 2,
            ftCenterDelta: (ft.left + ft.right) / 2 - altitude.left - altitudeWidth / 2,
            scaleHeightDelta: document.querySelector('#airspeed-tape-clip rect').getBBox().height -
              document.querySelector('#altitude-tape-clip rect').getBBox().height,
            backgroundHeightDelta: ias.height - altitude.height,
            footerHeightDelta: gs.height - baro.height,
            gaps: [gs.top - ias.bottom, baro.top - altitude.bottom, panel.bottom - gs.bottom,
              panel.bottom - baro.bottom, ias.left - panel.left, panel.right - altitude.right],
            gsLabelStyle: style('.ground-speed-label'), gsValueStyle: style('.ground-speed-value'),
            extras: document.querySelectorAll('.ground-speed-unit, .barometric-unit').length,
            edgesCovered,
          };
        });
        assert.deepEqual(geometry.unitTexts, ['[KT]', '[FT]']);
        assert.deepEqual(geometry.ktStyle, geometry.ftStyle);
        for (const key of ['labelDelta', 'ktCenterDelta', 'ftCenterDelta', 'scaleHeightDelta',
          'backgroundHeightDelta', 'footerHeightDelta']) assert.ok(Math.abs(geometry[key]) < 0.1, `${key}: ${JSON.stringify(geometry)}`);
        assert.ok(geometry.gaps.every(g => Math.abs(g) < 0.01));
        assert.deepEqual(geometry.gsLabelStyle, geometry.gsValueStyle);
        assert.equal(geometry.extras, 0);
        assert.equal(geometry.edgesCovered, true);
      }
      await page.locator('main').evaluate(n => { n.style.padding = '3px'; });
      await page.waitForFunction(() => {
        const svg = document.querySelector('svg'), b = svg.getBoundingClientRect();
        return Math.abs(Number(svg.getAttribute('viewBox').split(' ')[2]) - b.width) < 0.01;
      });
      for (const pitch of [-90, 90]) {
        for (const text of ['23.83 inHg', '1026 hPa', 'BARO', '', '  BARO  ', '<b>BARO</b>', 'X', 'OVR']) {
          setFixture({ baro_text: text, pitch_deg: pitch });
          await page.waitForFunction(({ text, pitch }) => document.querySelector('#barometric-value').textContent === text &&
            Math.sign(document.querySelector('#world-pitch').transform.baseVal.consolidate().matrix.f) === Math.sign(pitch), { text, pitch });
          const actual = await page.evaluate(() => {
            const n = document.querySelector('#barometric-value'), text = n.getBoundingClientRect();
            const background = document.querySelector('.barometric-background').getBoundingClientRect();
            const s = getComputedStyle(n);
            const anchor = new DOMPoint(Number(n.getAttribute('x')), Number(n.getAttribute('y'))).matrixTransform(n.getScreenCTM());
            return { text: n.textContent, children: n.children.length, label: n.parentElement.getAttribute('aria-label'),
              state: n.parentElement.dataset.state, style: [s.fontSize, s.fontWeight, s.fill],
              anchorDelta: anchor.x - (background.left + background.right) / 2,
              inside: !n.textContent || (text.left >= background.left + 1 && text.right <= background.right - 1 &&
                text.top >= background.top + 1 && text.bottom <= background.bottom - 1),
              centerDelta: !n.textContent ? 0 : (text.left + text.right - background.left - background.right) / 2 };
          });
          assert.equal(actual.text, text); assert.equal(actual.children, 0);
          assert.equal(actual.label, `Barometric setting ${text}`); assert.equal(actual.state, 'available');
          assert.deepEqual(actual.style, ['11px', '600', 'rgb(134, 222, 245)']);
          assert.ok(Math.abs(actual.anchorDelta) < 0.01, JSON.stringify({ width, height, actual }));
          // Glyph side bearings can differ from the centered advance width by a fraction of a pixel.
          if (['23.83 inHg', '1026 hPa', 'BARO'].includes(text)) {
            assert.ok(actual.inside && Math.abs(actual.centerDelta) < 0.5, JSON.stringify({ width, height, actual }));
          }
          if (['23.83 inHg', '1026 hPa', 'BARO'].includes(text)) {
            await page.screenshot({ path: `.local/pfd-browser/footer-${path === '/' ? 'production' : 'layout'}-${width}x${height}-${pitch}-${text.includes('inHg') ? 'inhg' : text.includes('hPa') ? 'hpa' : 'baro'}.png` });
          }
        }
      }
      for (const speed of [0, 9, 95, 157, 999, null, 1000]) {
        setFixture({ gs_kt: speed });
        await page.waitForFunction(speed => document.querySelector('#ground-speed').dataset.state ===
          (speed === null ? 'unavailable' : speed > 999 ? 'overflow' : 'available') &&
          document.querySelector('#ground-speed-value').textContent === (speed === null || speed > 999 ? '' : String(speed)), speed);
        assert.ok(await page.evaluate(() => {
          const label = document.querySelector('.ground-speed-label').getBoundingClientRect();
          const group = document.querySelector('#ground-speed');
          const reading = document.querySelector(group.dataset.state === 'available' ? '#ground-speed-value' : '#ground-speed-warning').getBoundingClientRect();
          const background = group.querySelector('rect').getBoundingClientRect();
          return label.right < reading.left && reading.right <= background.right && label.left >= background.left;
        }), `GS collision at ${speed}, ${width}x${height}`);
      }
    }
    for (const bad of [null, 29.92, false, {}, []]) {
      setFixture({ baro_text: bad });
      await page.locator('#barometric-setting[data-state="unavailable"]').waitFor();
      assert.equal(await page.locator('#barometric-value').textContent(), 'X');
    }
    // JSON omission in the fixture follows the same path as an absent source field.
    setFixture({ baro_text: null, omit_baro: true });
    await page.locator('#barometric-setting[data-state="unavailable"]').waitFor();
    setFixture({ omit_baro: false, baro_text: 'BARO', gs_kt: 157, pitch_deg: 0 });
    await page.waitForFunction(() => document.querySelector('#barometric-value').textContent === 'BARO');
  }
  setFixture({ baro_text: '29.92 inHg', pitch_deg: 0, roll_deg: 0, gs_kt: 137, altitude_ft: 10000 });
  await page.goto(url);
  await page.locator('.instrument[data-available="true"]').waitFor();
  await page.evaluate(() => { window.initialInstrumentNodes = [...document.querySelectorAll('svg *')]; });
}

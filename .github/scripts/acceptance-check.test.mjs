import test from 'node:test';
import assert from 'node:assert/strict';
import { CONTEXT, closingIssues, validateAcceptance, githubApi, runAcceptance } from './acceptance-check.mjs';

const repo = 'example/pfd';
const sha = 'a'.repeat(40);
const criterion = 'Check GS on a real phone.';
const issue = (body = `## Acceptance criteria\n- [x] ${criterion}`, number = 21) => ({ number, body });
const pr = (body = `Closes #21\n\n## Additional verification\n- [x] ${criterion}`, number = 47, head = sha) =>
  ({ number, state: 'open', body, base: { ref: 'main' }, head: { sha: head } });

test('closing references support GitHub keywords, case, same-repository URLs and qualified numbers', () => {
  assert.deepEqual(closingIssues('Closes #21. FIXED example/pfd#22. Resolves https://github.com/example/pfd/issues/23. Fixes #21.', repo),
    { numbers: [21, 22, 23], errors: [] });
  assert.match(closingIssues('Closes other/repo#21', repo).errors.join(), /this repository/);
  assert.deepEqual(closingIssues('Refs #21', repo), { numbers: [], errors: [] });
  assert.match(closingIssues('Closes #0', repo).errors.join(), /Invalid issue/);
  assert.equal(closingIssues('Example: `Closes #21`', repo).numbers.length, 0);
  assert.deepEqual(closingIssues('## Related issue\nCloses #21', repo), { numbers: [21], errors: [] });
});

test('hidden, quoted and fenced tasks do not affect optional acceptance or verification', () => {
  const hidden = `<!-- Closes #21 -->\n\`\`\`markdown\nCloses #21\n\`\`\``;
  assert.equal(closingIssues(hidden, repo).numbers.length, 0);
  const body = `Closes #21\n## Additional verification\n> - [x] ${criterion}\n~~~\n- [x] ${criterion}\n~~~\n<!-- - [x] ${criterion} -->`;
  assert.deepEqual(validateAcceptance(body, [issue()]), []);
  assert.deepEqual(validateAcceptance('', [issue('## Acceptance criteria\n> - [ ] Example')]), []);
  assert.deepEqual(validateAcceptance('Closes #21', [issue('## Acceptance criteria\n```\n- [ ] Done\n```')]), []);
});

test('issue acceptance and PR checklists must both be checked, without comparing wording', () => {
  assert.deepEqual(validateAcceptance(pr().body, [issue()]), []);
  assert.match(validateAcceptance(pr().body, [issue(`## Acceptance criteria\n- [ ] ${criterion}`)]).join(), /Unchecked acceptance/);
  assert.match(validateAcceptance(pr().body.replace('[x]', '[ ]'), [issue()]).join(), /Unchecked PR task/);
  assert.deepEqual(validateAcceptance(pr().body.replace(criterion, 'Different contextual wording.'), [issue()]), []);
  assert.match(validateAcceptance(`${pr().body}\n## Tests\n- [ ] Run another check.`, [issue()]).join(), /Run another check/);
});

test('only the issue Acceptance criteria section is checked; PR tasks are checked everywhere', () => {
  const source = issue(`### Acceptance criteria\n* [X] Automated geometry.\n### Real devices\n- [ ] ${criterion}`);
  assert.deepEqual(validateAcceptance(pr().body, [source]), []);
  assert.deepEqual(validateAcceptance('Closes #21\n## Additional verification\nNone.', [source]), []);
  assert.match(validateAcceptance('Closes #21\n## Tests\n- [ ] A PR check.', [source]).join(), /Unchecked PR/);
});

test('issues need no checklist; closing references to PRs still fail', () => {
  assert.deepEqual(validateAcceptance('Closes #21', [issue('## Acceptance criteria\n- [x] Automated test passed.')]), []);
  assert.deepEqual(validateAcceptance('Closes #21', [issue('No criteria yet.')]), []);
  assert.match(validateAcceptance('Closes #21', [{ ...issue(), pull_request: {} }]).join(), /not an issue/);
});

test('wording, duplicates and PR section names do not need to match the issue', () => {
  assert.deepEqual(validateAcceptance('## Additional verification\n1. [X] Check   GS on a real phone.', [issue()]), []);
  assert.deepEqual(validateAcceptance('## Verification\n- [x] Different check.', [issue(), issue(undefined, 22)]), []);
  assert.deepEqual(validateAcceptance(`${pr().body}\n- [x] ${criterion}`, [issue(), issue(undefined, 22)]), []);
});

test('nested and repeated acceptance headings include every criterion', () => {
  assert.match(validateAcceptance('', [issue('## Acceptance criteria\n- [x] Done\n### Devices\n- [ ] Phone')]).join(), /Phone/);
  assert.match(validateAcceptance('', [issue('## Acceptance criteria\n- [x] Done\n## Notes\nText\n## Acceptance Criteria\n- [ ] Second')]).join(), /Second/);
  assert.deepEqual(validateAcceptance(`## Additional verification\n### Phone\n- [x] ${criterion}`, [issue()]), []);
});

test('literal comments in code cannot hide unchecked tasks; malformed fence endings remain code', () => {
  const body = 'Closes #21\n```text\n<!--\n```\n- [ ] Visible incomplete check.';
  assert.match(validateAcceptance(body, [issue()]).join(), /Visible incomplete/);
  assert.deepEqual(validateAcceptance('', [issue('## Acceptance criteria\n```\nexample\n```still-code\n- [ ] Hidden')]), []);
  assert.deepEqual(validateAcceptance('', [issue('## Acceptance criteria\n\n    - [ ] Code example')]), []);
  assert.deepEqual(validateAcceptance('- [x] Parent\n    - [x] Nested', [issue()]), []);
  assert.match(validateAcceptance('- [x] Parent\n    - [ ] Nested', [issue()]).join(), /Nested/);
});

test('multiline wording does not matter; the checkbox state controls acceptance', () => {
  assert.deepEqual(validateAcceptance('- [x] Short wording', [issue('## Acceptance criteria\n- [x] Verify GS\n  on a real phone.')]), []);
  assert.match(validateAcceptance('- [x] Short wording', [issue('## Acceptance criteria\n- [ ] Verify GS\n  on a real phone.')]).join(), /Unchecked acceptance/);
});

test('inline code comments, nested fenced examples and task continuation lines obey GFM structure', () => {
  assert.match(validateAcceptance('Closes #21\nExample: `<!--`\n- [ ] Visible incomplete check.', [issue()]).join(), /Visible incomplete/);
  assert.deepEqual(validateAcceptance('', [issue('## Acceptance criteria\n- Example:\n    ```markdown\n    - [ ] Hidden example\n    ```')]), []);
  assert.match(validateAcceptance('Closes #21\n- [ ]\n  Visible incomplete check.', [issue()]).join(), /Visible incomplete/);
  assert.match(validateAcceptance('', [issue('## Acceptance criteria\n- [ ]\n  Continue here.')]).join(), /Continue here/);
  assert.deepEqual(validateAcceptance('Closes #21\n- [x]\n  Done.', [issue()]), []);
});

function fakeApi(initial = [pr()]) {
  const state = { prs: initial, issues: new Map([[21, issue()]]), statuses: [], writes: [], reads: [] };
  state.api = async (path, method = 'GET', body) => {
    state.reads.push(path);
    if (method === 'POST') { state.writes.push({ path, body }); state.statuses.unshift({ ...body, sha: path.split('/').at(-1) }); return body; }
    if (path.includes('/pulls?')) return state.prs;
    if (path.includes('/pulls/')) return state.prs.find(p => p.number === Number(path.split('/').at(-1)));
    if (path.includes('/issues/')) {
      const value = state.issues.get(Number(path.split('/').at(-1)));
      if (!value) throw new Error('GitHub API GET issue failed (404).');
      return value;
    }
    if (path.includes('/commits/')) return state.statuses.filter(status => path.includes(`/commits/${status.sha}/`));
    throw new Error(`Unexpected path ${path}`);
  };
  return state;
}
const run = state => runAcceptance({ api: state.api, repository: repo, targetUrl: 'https://github.com/example/pfd/actions/runs/1' });

test('PRs without closing references or checklists publish success without issue API requests', async () => {
  for (const body of ['', 'Describe the change.', 'Refs #21', 'Example: `Closes #21`']) {
    assert.deepEqual(closingIssues(body, repo), { numbers: [], errors: [] });
    const state = fakeApi([pr(body)]);
    assert.equal((await run(state))[0].state, 'success');
    assert.equal(state.writes[0].body.context, CONTEXT);
    assert.ok(state.writes[0].path.endsWith(sha));
    assert.ok(!state.reads.some(path => path.includes('/issues/')));
    await run(state);
    assert.equal(state.writes.length, 1);
  }
});

test('linked issues can omit a checklist or contain an empty acceptance section', async () => {
  for (const body of [null, '', 'No checklist.', '## Acceptance criteria\nExpected behavior in prose.', '## Acceptance criteria\n## Notes\n- [ ] Unrelated']) {
    const state = fakeApi();
    state.issues.set(21, issue(body));
    assert.equal((await run(state))[0].state, 'success');
    assert.ok(state.reads.some(path => path.includes('/issues/21')));
  }
});

test('visible PR tasks remain enforced without issue references', async () => {
  const state = fakeApi([pr('- [ ] Pending local check')]);
  assert.equal((await run(state))[0].state, 'failure');
  assert.match((await run(state))[0].errors.join(), /Pending local check/);
  state.prs[0].body = '- [x] Local check completed';
  assert.equal((await run(state))[0].state, 'success');
});

test('hidden unchecked PR tasks are ignored even without an issue', async () => {
  const body = '> - [ ] Quoted example\n\n```markdown\n- [ ] Fenced example\n```\n\n<!-- - [ ] Comment -->\n\n    - [ ] Indented example';
  assert.equal((await run(fakeApi([pr(body)])))[0].state, 'success');
  assert.equal((await run(fakeApi([pr(`${body}\n\n- [ ] Visible task`)])))[0].state, 'failure');
});

test('optional issue criteria become blocking when an unchecked checklist is supplied', async () => {
  const state = fakeApi();
  state.issues.set(21, issue('No checklist.'));
  assert.equal((await run(state))[0].state, 'success');
  state.issues.set(21, issue('## Acceptance criteria\n- [ ] New criterion'));
  assert.equal((await run(state))[0].state, 'failure');
});

test('unlinked PRs sharing a head cannot override an incomplete linked checklist', async () => {
  const state = fakeApi([pr('No issue.', 47), pr('Closes #21', 49)]);
  state.issues.set(21, issue('## Acceptance criteria\n- [ ] Required check'));
  assert.equal((await run(state))[0].state, 'failure');
  assert.equal(state.writes.length, 1);
});

test('unlinked PR metadata and shared-head membership are rechecked before success', async () => {
  for (const edit of ['body', 'membership']) {
    const state = fakeApi([pr('No issue.')]);
    const api = state.api;
    let edited = false;
    state.api = async (path, method, body) => {
      const result = structuredClone(await api(path, method, body));
      if (!edited && path.includes('/commits/')) {
        edited = true;
        if (edit === 'body') state.prs[0].body += '\n- [ ] Newly incomplete';
        else state.prs.push(pr('- [ ] Another PR task', 49));
      }
      return result;
    };
    assert.equal((await run(state))[0].state, 'failure', edit);
  }
});

test('API integration publishes success on the current PR head and skips identical scheduled results', async () => {
  const state = fakeApi();
  const result = await run(state);
  assert.equal(result[0].state, 'success');
  assert.equal(state.writes[0].path, `/repos/${repo}/statuses/${sha}`);
  assert.equal(state.writes[0].body.context, CONTEXT);
  await run(state);
  assert.equal(state.writes.length, 1);
});

test('PR edits, issue edits and new heads invalidate earlier success without a new code change', async () => {
  const state = fakeApi();
  await run(state);
  state.prs[0].body = 'Closes #21\n- [ ] Newly incomplete';
  assert.equal((await run(state))[0].state, 'failure');
  state.prs[0] = pr();
  assert.equal((await run(state))[0].state, 'success');
  state.issues.set(21, issue(`## Acceptance criteria\n- [ ] ${criterion}\n- [ ] A new check.`));
  assert.equal((await run(state))[0].state, 'failure');
  state.prs[0].head.sha = 'b'.repeat(40);
  await run(state);
  assert.ok(state.writes.at(-1).path.endsWith('b'.repeat(40)));
});

test('invalid references and issue API errors cannot produce success', async () => {
  const state = fakeApi([pr('Closes other/repo#21')]);
  assert.equal((await run(state))[0].state, 'failure');
  state.prs = [pr('Closes #999')];
  assert.equal((await run(state))[0].state, 'error');
  assert.equal(state.writes.at(-1).body.state, 'error');
  const event = { pull_request: pr() };
  const writes = [];
  await assert.rejects(runAcceptance({ repository: repo, event, api: async (path, method, body) => {
    if (method === 'POST') { writes.push(body); return {}; }
    throw new Error('API unavailable');
  } }), /API unavailable/);
  assert.deepEqual(writes.map(w => w.state), ['pending', 'error']);
});

test('PRs sharing a head cannot overwrite a failing checklist with a passing status', async () => {
  const state = fakeApi([pr(), pr('Closes #21\n- [ ] Pending check', 49)]);
  assert.equal((await run(state))[0].state, 'failure');
  assert.equal(state.writes.length, 1);
  assert.match((await run(state))[0].errors.join(), /PR #49/);
});

test('a head or base changing during validation never receives a success result', async () => {
  const state = fakeApi();
  const api = state.api;
  state.api = async (path, method, body) => path.includes('/pulls/47') ? pr(undefined, 47, 'b'.repeat(40)) : api(path, method, body);
  assert.equal((await run(state))[0].state, 'failure');
});

test('PR list pagination validates later pages instead of silently dropping them', async () => {
  const state = fakeApi();
  const api = state.api;
  state.api = async (path, method, body) => path.includes('/pulls?') ?
    (path.endsWith('page=1') ? Array.from({ length: 100 }, () => pr()) : [pr('Closes #21\n- [ ] Pending check', 49)]) : api(path, method, body);
  state.prs.push(pr('Closes #21\n- [ ] Pending check', 49));
  assert.equal((await run(state))[0].state, 'failure');
});

test('a later PR-list page failure invalidates heads already discovered', async () => {
  const state = fakeApi();
  const api = state.api;
  state.api = async (path, method, body) => {
    if (path.includes('/pulls?')) {
      if (path.endsWith('page=1')) return Array.from({ length: 100 }, () => pr());
      throw new Error('Page 2 unavailable');
    }
    return api(path, method, body);
  };
  await assert.rejects(run(state), /Page 2 unavailable/);
  assert.equal(state.writes.at(-1).body.state, 'error');
});

test('PR body, issue body and shared-head membership edits during API reads cannot grant success', async () => {
  for (const edit of ['pr', 'issue', 'membership']) {
    const state = fakeApi();
    const api = state.api;
    let edited = false;
    state.api = async (path, method, body) => {
      const result = structuredClone(await api(path, method, body));
      if (!edited && path.includes('/issues/')) {
        edited = true;
        if (edit === 'pr') state.prs[0].body += '\n- [ ] Newly incomplete';
        if (edit === 'issue') state.issues.get(21).body += '\n- [ ] Newly incomplete';
        if (edit === 'membership') state.prs.push(pr('Closes #21\n- [ ] Newly incomplete', 49));
      }
      return result;
    };
    assert.equal((await run(state))[0].state, 'failure', edit);
  }
});

test('API requests use fixed GitHub origin, refuse redirects, and do not disclose token in errors', async () => {
  const api = githubApi('test-secret', async (url, options) => {
    assert.equal(url, 'https://api.github.com/repos/example/pfd/issues/21');
    assert.equal(options.redirect, 'error');
    assert.equal(options.headers.Authorization, 'Bearer test-secret');
    return { ok: false, status: 403 };
  });
  await assert.rejects(api('/repos/example/pfd/issues/21'), error =>
    error.message.includes('403') && !error.message.includes('test-secret'));
});

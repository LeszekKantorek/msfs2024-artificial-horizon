import { createHash } from 'node:crypto';
import { readFileSync, appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { lexer } from 'marked';

export const CONTEXT = 'Acceptance criteria';

const parse = body => lexer(String(body ?? ''), { gfm: true });
const excluded = new Set(['code', 'codespan', 'html', 'blockquote', 'image']);

// Read tokens only. Never render or execute HTML from issue or PR descriptions.
function visibleText(tokens, separator = '\n') {
  return tokens.map(token => {
    if (excluded.has(token.type)) return ' ';
    if (token.items) return token.items.map(item => visibleText(item.tokens)).join('\n');
    if (token.tokens) return visibleText(token.tokens, '');
    return token.text ?? ' ';
  }).join(separator);
}

function section(tokens, title) {
  let level;
  let found = false;
  const content = [];
  for (const token of tokens) {
    if (token.type === 'heading') {
      if (level && token.depth <= level) level = undefined;
      if (!level && visibleText(token.tokens).trim().toLowerCase() === title.toLowerCase()) {
        level = token.depth; found = true; continue;
      }
    }
    if (level) content.push(token);
  }
  return { found, tokens: content };
}

function tasks(tokens) {
  const result = [];
  for (const token of tokens) {
    if (excluded.has(token.type)) continue;
    for (const item of token.items ?? []) {
      // GFM permits task text on a continuation line, including an empty marker.
      const marker = item.text.match(/^\[([ xX])\](?:\s|$)/);
      if (item.task || marker) result.push({
        checked: item.task ? item.checked : marker[1].toLowerCase() === 'x',
        text: visibleText(item.tokens).replace(/^\[([ xX])\]\s*/, '').replace(/\s+/g, ' ').trim() || '(empty task)',
      });
      result.push(...tasks(item.tokens));
    }
    if (!token.items && token.tokens) result.push(...tasks(token.tokens));
  }
  return result;
}

export function closingIssues(body, repository) {
  const refs = [];
  const errors = [];
  const pattern = /\b(?:close[sd]?|fix(?:es|ed)?|resolve[sd]?)\s+:?\s*(https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+|[\w.-]+\/[\w.-]+#\d+|#\d+)\b/gi;
  for (const match of visibleText(parse(body)).matchAll(pattern)) {
    const reference = match[1];
    const number = Number(reference.match(/(?:#|\/)(\d+)$/)[1]);
    const repo = reference.startsWith('#') ? repository : reference.replace('https://github.com/', '').split(/#|\/issues\//)[0];
    if (repo.toLowerCase() !== repository.toLowerCase()) errors.push(`Closing reference ${reference} must name this repository.`);
    else if (!Number.isSafeInteger(number) || number < 1) errors.push(`Invalid issue number: ${reference}.`);
    else refs.push(number);
  }
  if (!refs.length) errors.push('Add a same-repository closing reference such as Closes #21.');
  return { numbers: [...new Set(refs)], errors };
}

export function validateAcceptance(prBody, issues) {
  const errors = [];
  const visible = parse(prBody);
  for (const issue of issues) {
    if (issue.pull_request) { errors.push(`#${issue.number} is a PR, not an issue.`); continue; }
    const body = parse(issue.body);
    const acceptance = section(body, 'Acceptance criteria');
    const criteria = tasks(acceptance.tokens);
    if (!criteria.length) errors.push(`#${issue.number} needs an Acceptance criteria section with task items.`);
    for (const task of criteria.filter(task => !task.checked)) {
      errors.push(`Unchecked acceptance criterion (#${issue.number}): ${task.text}`);
    }
  }
  for (const task of tasks(visible).filter(task => !task.checked)) errors.push(`Unchecked PR task: ${task.text}`);
  return errors;
}

export function githubApi(token, fetcher = fetch) {
  return async (path, method = 'GET', body) => {
    const response = await fetcher(`https://api.github.com${path}`, {
      method, redirect: 'error', signal: AbortSignal.timeout(15000),
      headers: { Accept: 'application/vnd.github+json', Authorization: `Bearer ${token}`,
        'X-GitHub-Api-Version': '2022-11-28', ...(body ? { 'Content-Type': 'application/json' } : {}) },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    if (!response.ok) throw new Error(`GitHub API ${method} ${path} failed (${response.status}).`);
    return response.json();
  };
}

export async function runAcceptance({ api, repository, baseBranch = 'main', targetUrl, event = {} }) {
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository)) throw new Error('Invalid repository name.');
  const prefix = `/repos/${repository}`;
  const report = [];
  const knownHeads = new Set();
  const publish = (sha, state, description) => api(`${prefix}/statuses/${sha}`, 'POST', {
    context: CONTEXT, state, description: description.slice(0, 140), target_url: targetUrl,
  });
  const listPrs = async () => {
    const prs = [];
    for (let page = 1; ; page++) {
      const batch = await api(`${prefix}/pulls?state=open&base=${encodeURIComponent(baseBranch)}&per_page=100&page=${page}`);
      if (!Array.isArray(batch)) throw new Error('Invalid PR list response.');
      for (const pr of batch) {
        if (!/^[a-f0-9]{40}$/.test(pr.head?.sha ?? '')) throw new Error('Invalid PR head response.');
        knownHeads.add(pr.head.sha);
      }
      prs.push(...batch);
      if (batch.length < 100) return prs;
    }
  };
  const eventHead = event.pull_request?.head?.sha;
  if (/^[a-f0-9]{40}$/.test(eventHead ?? '') && event.pull_request.base.ref === baseBranch) {
    knownHeads.add(eventHead);
    await publish(eventHead, 'pending', 'Checking current issue criteria and PR verification.');
  }
  try {
    const prs = await listPrs();
    // Commit statuses belong to a SHA, not a PR. Aggregate all PRs sharing a head.
    const groups = new Map();
    for (const pr of prs) {
      if (!groups.has(pr.head.sha)) groups.set(pr.head.sha, []);
      groups.get(pr.head.sha).push(pr);
    }
    for (const [sha, group] of groups) {
      const errors = [];
      const evidence = [];
      try {
        for (const listed of group) {
          const pr = await api(`${prefix}/pulls/${listed.number}`);
          if (pr.head.sha !== sha || pr.state !== 'open' || pr.base.ref !== baseBranch) {
            errors.push(`PR #${listed.number} changed during validation; run the check again.`); continue;
          }
          const refs = closingIssues(pr.body, repository);
          errors.push(...refs.errors.map(error => `PR #${pr.number}: ${error}`));
          const issues = [];
          for (const number of refs.numbers) issues.push(await api(`${prefix}/issues/${number}`));
          errors.push(...validateAcceptance(pr.body, issues).map(error => `PR #${pr.number}: ${error}`));
          evidence.push({ number: pr.number, body: pr.body, issues: issues.map(i => ({ number: i.number, body: i.body })) });
        }
        const statuses = await api(`${prefix}/commits/${sha}/statuses?per_page=100`);
        // Re-read before granting success: edits can occur while issue data loads.
        if (!errors.length) {
          const current = (await listPrs()).filter(pr => pr.head.sha === sha);
          const membership = items => items.map(pr => pr.number).sort((a, b) => a - b).join(',');
          if (membership(current) !== membership(group)) errors.push('PR membership changed during validation; run the check again.');
          for (const saved of evidence) {
            const pr = await api(`${prefix}/pulls/${saved.number}`);
            if (pr.head.sha !== sha || pr.state !== 'open' || pr.base.ref !== baseBranch || pr.body !== saved.body) {
              errors.push(`PR #${saved.number} changed during validation; run the check again.`);
            }
            for (const savedIssue of saved.issues) {
              const issue = await api(`${prefix}/issues/${savedIssue.number}`);
              if (issue.body !== savedIssue.body || issue.pull_request) errors.push(`#${savedIssue.number} changed during validation; run the check again.`);
            }
          }
        }
        const fingerprint = createHash('sha256').update(JSON.stringify(evidence)).digest('hex').slice(0, 12);
        const state = errors.length ? 'failure' : 'success';
        const description = `${errors.length ? `${errors.length} acceptance problem(s)` : 'All acceptance checks confirmed'} [${fingerprint}]`;
        const latest = statuses.find(status => status.context === CONTEXT);
        if (latest?.state !== state || latest?.description !== description) await publish(sha, state, description);
        report.push({ sha, prs: group.map(pr => pr.number), state, errors });
      } catch (error) {
        await publish(sha, 'error', 'Acceptance validation could not read current GitHub data.');
        report.push({ sha, prs: group.map(pr => pr.number), state: 'error', errors: [error.message] });
      }
    }
  } catch (error) {
    for (const sha of knownHeads) await publish(sha, 'error', 'Acceptance validation failed; retry the workflow.');
    throw error;
  }
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (!process.env.GITHUB_TOKEN) throw new Error('GITHUB_TOKEN is required.');
    const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
    const report = await runAcceptance({ api: githubApi(process.env.GITHUB_TOKEN),
      repository: process.env.GITHUB_REPOSITORY, event,
      targetUrl: `https://github.com/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}` });
    for (const result of report) {
      console.log(`PR ${result.prs.map(number => `#${number}`).join(', ')}: ${result.state}`);
      for (const error of result.errors) console.log(`  - ${error}`);
    }
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY,
      `## Acceptance validation\n\n${report.map(r => `* PR ${r.prs.join(', ')}: **${r.state}**\n${r.errors.map(e => `  * ${e}`).join('\n')}`).join('\n')}\n`);
    if (report.some(result => result.state !== 'success')) process.exitCode = 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}

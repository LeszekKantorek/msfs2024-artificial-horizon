import { launchOwned } from './process.mjs';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
const dir = await mkdtemp(path.join(tmpdir(), 'live timer '));
const app = await launchOwned({ exe: process.env.S1_EXE, source: 'demo', runDirectory: dir });
await app.stop();
await rm(dir, { recursive: true, force: true });

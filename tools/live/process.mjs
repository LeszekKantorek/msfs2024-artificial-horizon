import {spawn} from 'node:child_process';

const wait = (promise, ms) => new Promise(resolve => { const timer = setTimeout(() => resolve(false), ms); promise.then(() => {clearTimeout(timer); resolve(true);}); });
export async function launchOwned({exe, appArgs}) {
  const child = spawn(exe, appArgs, {shell:false, windowsHide:true, stdio:['pipe','pipe','pipe']});
  let exit, stdinError, stopped;
  const childStopped = new AbortController();
  child.stdin.on('error', error => { stdinError = error.message; });
  const exited = new Promise(resolve => {
    child.once('error', error => { exit = {code:null,error:error.message}; childStopped.abort(); resolve(exit); });
    child.once('exit', (code, sig) => { exit = {code,signal:sig}; childStopped.abort(); resolve(exit); });
  });
  child.stderr.pipe(process.stderr, {end:false});
  child.stdout.pipe(process.stdout, {end:false});
  const stop = () => stopped ??= (async () => {
    let error = null, forced = false;
    if (exit) error = 'Application exited before shutdown was requested';
    if (!exit) {
      try { child.stdin.end('stop\n'); } catch (e) { error = e.message; }
      if (!await wait(exited, 10000)) {
        forced = true;
        if (!child.kill()) error = 'Failed to terminate owned child';
        if (!await wait(exited, 5000)) error = 'Owned child did not exit';
      }
    }
    return {exitCode:exit?.code ?? null,forced,error:error ?? stdinError ?? exit?.error ?? null};
  })();
  return {stop, signal:childStopped.signal};
}

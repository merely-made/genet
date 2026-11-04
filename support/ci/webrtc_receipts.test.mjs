// Regression for durable run identity and resumption, independent of browsers.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, writeFile, readFile, unlink } from 'node:fs/promises';
import path from 'node:path';
import { RestartJournal } from './webrtc_receipts.mjs';

test('run metadata preserves its record type and can resume', async()=>{
  const directory=path.resolve('C:/t/cargo-targets/genet/rtc-interop');
  const file=path.join(directory,'journal-fixture.md');
  const marker='# Owned Genet RTC journal regression fixture; generated, not qualification evidence.\n';
  await mkdir(directory,{recursive:true});
  await writeFile(file,marker,{flag:'wx'});
  try {
    const identity={gate:'E2d',dns:false,expectedCases:1,choices:{
      browser:['chrome'],engine:['boa'],route:['direct'],offerer:['native'],cleanup:['close']
    }};
    const first=await RestartJournal.open(file,identity,'fixture-source');
    const rows=()=>readFile(file,'utf8').then(text=>[...text.matchAll(/```json\n(.*?)\n```/gs)].map(m=>JSON.parse(m[1])));
    assert.equal((await rows())[0].gate,'E2d-run');
    const resumed=await RestartJournal.open(file,identity,'fixture-source','latest');
    assert.equal(resumed.run,first.run);
    assert.equal((await rows())[1].gate,'E2d-resume');
    await assert.rejects(RestartJournal.open(file,{...identity,expectedCases:2},'fixture-source','latest'),/identical source/);
    await resumed.finish('simulated interruption',true);
    const end=(await rows()).at(-1);
    assert.equal(end.gate,'E2d-run-end');assert.equal(end.completeSelectedMatrix,false);
  } finally {
    assert.equal(path.dirname(path.resolve(file)),directory);
    assert.ok((await readFile(file,'utf8')).startsWith(marker));
    await unlink(file);
  }
});

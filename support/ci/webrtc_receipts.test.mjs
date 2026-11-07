// Regression for durable run identity and resumption, independent of browsers.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, writeFile, readFile, unlink } from 'node:fs/promises';
import path from 'node:path';
import { RestartJournal, assertWritablePacket } from './webrtc_receipts.mjs';

test('both acceptance and unresolved-gate audit closures freeze packets',()=>{
  for(const marker of ['Packet closed after the final qualified run','Packet closed after the final audit'])
    assert.throws(()=>assertWritablePacket(marker),/packet is closed/);
  assertWritablePacket('In progress; unsuccessful cases retained.');
});

test('advertisement checkpoints require named-host selection and restart proof',()=>{
  const journal=new RestartJournal();
  journal.identity={localMdns:true,choices:{browser:['chrome'],engine:['boa'],route:['direct'],offerer:['native'],cleanup:['close']}};
  const row={browser:'chrome',engine:'boa',route:'direct',offerer:'native',cleanup:'close',status:'pass',
    advertisement:{private:true,hostCount:1,browserNamedHostSelected:true},
    restarts:[{advertisement:{private:true,nameStable:true}},{advertisement:{private:true,nameStable:true}}]};
  journal.validateCase(row);
  assert.throws(()=>journal.validateCase({...row,advertisement:undefined}),/privacy evidence/);
  assert.throws(()=>journal.validateCase({...row,advertisement:{...row.advertisement,browserNamedHostSelected:false}}),/not selected/);
  assert.throws(()=>journal.validateCase({...row,restarts:[]}),/restart advertisement/);
});

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
    await resumed.case({browser:'chrome',engine:'boa',route:'direct',offerer:'native',cleanup:'close',status:'fail',phase:'restart',error:'restart peer error',
      restartDiagnostics:{snapshots:{native:{received:[],channelStates:[],errors:['restart option:OperationError:ICE']}}}},true);
    assert.deepEqual((await rows()).at(-1).diagnostics.peers.native.errors,['restart option:OperationError:ICE']);
    await resumed.finish('simulated interruption',true);
    const end=(await rows()).at(-1);
    assert.equal(end.gate,'E2d-run-end');assert.equal(end.completeSelectedMatrix,false);
    await writeFile(file,'\nPacket closed after the final qualified run and restoration checks.\n',{flag:'a'});
    const frozen=await readFile(file,'utf8');
    await assert.rejects(RestartJournal.open(file,identity,'fixture-source'),/packet is closed/);
    await assert.rejects(resumed.append('resume',{}),/packet is closed/);
    assert.equal(await readFile(file,'utf8'),frozen);
  } finally {
    assert.equal(path.dirname(path.resolve(file)),directory);
    assert.ok((await readFile(file,'utf8')).startsWith(marker));
    await unlink(file);
  }
});

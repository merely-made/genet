import test from 'node:test';
import assert from 'node:assert/strict';
import {RestartSignals,summarizeSignal} from './webrtc_restart_signals.mjs';
const name='53caef04-4218-4fb0-815d-f91a511bb8d2.local';
const candidate={candidate:{candidate:`candidate:f 1 udp 100 ${name} 40000 typ host`,usernameFragment:'secret-ufrag'},generation:'secret-ufrag'};
const description={description:{type:'offer',sdp:`v=0\r\na=ice-ufrag:secret-ufrag\r\na=ice-pwd:secret-password\r\na=ice-options:trickle\r\na=${candidate.candidate.candidate}\r\na=end-of-candidates\r\n`}};
test('observations preserve signaling and exclude names, addresses and credentials',()=>{
  const d=new RestartSignals('observe');
  assert.equal(d.map('native',description,'offer'),description);
  assert.equal(d.map('native',candidate,'gathering'),candidate);
  assert.equal(d.map('native',{candidate:null},'gathering').candidate,null);
  assert.deepEqual(summarizeSignal(description),{type:'offer',candidates:1,namedHosts:1,numericHosts:0,components:[1],ends:1,trickle:true});
  const text=JSON.stringify(d.summary());for(const secret of [name,'secret-ufrag','secret-password'])assert(!text.includes(secret));
  for(let i=0;i<100;i++)d.map('native',candidate,'bounded');
  assert.equal(d.entries.length,64);assert(d.omitted>0);
});
test('deferred end markers keep exact candidate bytes and never alter browser signals',()=>{
  const d=new RestartSignals('defer-end');
  assert.equal(d.map('browser',description,'offer'),description);
  assert.equal(d.map('native',candidate,'candidate'),candidate);
  const mapped=d.map('native',description,'offer');
  assert(mapped.description.sdp.includes(candidate.candidate.candidate));assert(!mapped.description.sdp.includes('end-of-candidates'));
  assert.equal(d.map('native',{candidate:null},'end'),null);
  assert.deepEqual(d.flush(),[{candidate:null}]);assert.equal(d.summary().pendingEnds,0);
});
test('trickle control removes embedded candidates but preserves actual trickled values',()=>{
  const d=new RestartSignals('trickle');
  const mapped=d.map('native',description,'offer');
  assert(!mapped.description.sdp.includes('a=candidate:'));assert(!mapped.description.sdp.includes('end-of-candidates'));
  assert.equal(d.map('native',candidate,'candidate'),candidate);
  assert.equal(description.description.sdp.includes(name),true);
});

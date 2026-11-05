import test from 'node:test';
import assert from 'node:assert/strict';
import {resolutionAnswer} from './webrtc_firefox_resolution_probe.mjs';
import {observeNames} from './webrtc_mdns_observer.mjs';
test('controlled answer is a bounded authoritative positive owned A record',()=>{
  const name='53caef04-4218-4fb0-815d-f91a511bb8d2.local';const answer=resolutionAnswer(name,'192.0.2.1');
  assert(answer.length<128);assert.deepEqual(observeNames(answer,new Set([name]),'192.0.2.1'),{queries:0,answers:1});
  assert.equal(answer.readUInt16BE(4),0);assert.equal(answer.readUInt16BE(6),1);
  assert.throws(()=>resolutionAnswer('unowned.local','192.0.2.1'));assert.throws(()=>resolutionAnswer(name,'::1'));
});

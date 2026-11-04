import test from 'node:test';
import assert from 'node:assert/strict';
import { settledStats } from './webrtc_stats.mjs';

function peer(states,extra={}) {
  let reads=0;
  return {evaluate:async()=>{},value:async()=>({pair:states[Math.min(reads++,states.length-1)]===null?null:{state:states[Math.min(reads-1,states.length-1)]},candidateReferencesValid:true,remoteAddressesRedacted:true,...extra})};
}
test('initial stats are resampled until both selected pairs succeed',async()=>{
  const result=await settledStats(peer([null,'succeeded']),peer(['in-progress','succeeded']),{pollMs:1});
  assert.equal(result.a.pair.state,'succeeded');assert.equal(result.b.pair.state,'succeeded');
  assert.deepEqual(result.settling,{attempts:2,pairStates:[[null,'in-progress'],['succeeded','succeeded']]});
});
test('missing selected pairs fail within a deadline and retain bounded states',async()=>{
  await assert.rejects(settledStats(peer([null]),peer(['succeeded']),{timeoutMs:30,pollMs:1}),error=>{
    assert.match(error.message,/statistics deadline/);
    assert.ok(error.initialStatsDiagnostics.attempts>=1);
    assert.deepEqual(error.initialStatsDiagnostics.pairStates,[[null,'succeeded']]);return true;
  });
});
test('settling never accepts an invalid native stats graph',async()=>{
  await assert.rejects(settledStats(peer(['succeeded'],{candidateReferencesValid:false}),peer(['succeeded'])),/graph or privacy/);
});

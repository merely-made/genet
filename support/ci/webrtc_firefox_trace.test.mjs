import test from 'node:test';
import assert from 'node:assert/strict';
import {FirefoxTrace} from './webrtc_firefox_trace.mjs';
test('Firefox log reduction retains fixed counters only across chunk boundaries',()=>{
  const trace=new FirefoxTrace();
  trace.push('mdns_service Query');trace.push('ing private-name.local\nmDNS response: private-name.local 192.0.2.1\nAddIceCandidate: private-name.local secret-ufrag\nICE stream failed\n');
  const view=trace.snapshot();assert.equal(view.mdnsQueries,1);assert.equal(view.mdnsResponses,1);assert.equal(view.remoteNamedCandidateCalls,1);assert.equal(view.iceFailures,1);
  const text=JSON.stringify(view);for(const secret of ['private-name','192.0.2.1','secret-ufrag'])assert(!text.includes(secret));
  trace.clear();assert.equal(trace.snapshot().lines,0);
});
test('oversized lines and partial buffers remain bounded',()=>{
  const trace=new FirefoxTrace();trace.push('a'.repeat(10000));assert.equal(trace.partial,'');
  trace.push('b'.repeat(10000)+'\n');assert.equal(trace.snapshot().lines,0);assert.equal(trace.snapshot().oversized,2);
});

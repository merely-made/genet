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
test('platform DNS evidence is limited to watched names and fixed words',()=>{
  const trace=new FirefoxTrace();trace.watch(['private-name.local']);
  trace.push('D/nsHostResolver resolving private-name.local cache miss\nD/nsHostResolver private-name.local completed negative cache\nD/nsHostResolver other.local failed\n');
  const view=trace.snapshot();assert.equal(view.nativeDnsLines,2);assert(view.dnsLabels.some(w=>w.includes('negative')));
  for(const secret of ['private-name.local','other.local'])assert(!JSON.stringify(view).includes(secret));
});

test('lookup completion and generation parser traces retain only fixed events',()=>{
  const trace=new FirefoxTrace();trace.watch(['private-name.local']);
  trace.pushStdout('Timecard created 1234.123456\n 0.001234 | 0.001234 | Set Remote Description | PeerConnectionImpl.cpp:1713 | SetRemoteDescription\n 0.002000 | 0.000766 | Look up mDNS name | file:1 | function\n');
  trace.pushStdout(' 0.003000 | 0.001000 | Done looking up mDNS name | file:1 | function\n 0.004000 | 0.001000 | private-name.local secret-ufrag | file:1 | function\n');
  trace.push('mtransport parsing trickle candidate private-name.local secret-ufrag\nmtransport Couldn\'t process ICE candidate private-name.local\nmtransport parsing trickle candidate other.local\n');
  const view=trace.snapshot();assert.equal(view.timecards,1);assert.deepEqual(view.timecardEvents.map(x=>x.event),['remote-description','lookup','resolved-forwarded']);assert.deepEqual(view.transportEvents,['parse','process-failed']);
  for(const secret of ['private-name','other.local','secret-ufrag','1234.123456'])assert(!JSON.stringify(view).includes(secret));
  for(let i=0;i<200;i++)trace.pushStdout(' 0.003000 | 0.001000 | Look up mDNS name | file:1 | function\n');
  assert.equal(trace.snapshot().timecardEvents.length,128);assert(trace.snapshot().timecardOmitted>0);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import {FirefoxPairOrder, holdBrowserSignal} from './webrtc_firefox_pair_order.mjs';

test('ordering trace links the queued check and nomination without retaining addresses or IDs', () => {
  const trace = new FirefoxPairOrder({address:'192.0.2.1',port:50000});
  const watched = new Set(['private.local']);
  trace.push('ICE(secret)/STREAM(private-stream)/COMP(1): Enqueuing STUN request pre-answer from IP4:192.0.2.1:50000/UDP', watched);
  trace.record('before-answer');
  trace.push('ICE-PEER(secret)/STREAM(private-stream)/COMP(1)/CAND-PAIR(private-pair): nominated pair is private-pair|IP4:192.0.2.2:50001/UDP|IP4:192.0.2.1:50000/UDP(host|prflx)', watched);
  trace.push('ICE(secret): peer (secret) parsing trickle ICE candidate private.local', watched);
  trace.push('ICE(secret): peer (secret), stream(private-stream) tried to trickle ICE in inappropriate state 4', watched);
  const rows = trace.snapshot().events;
  assert.deepEqual(rows.map(e=>e.event),['queued-check','before-answer','nominated','named-ice-entry','trickle-state-refusal']);
  assert.equal(rows[0].stream,rows[2].stream);
  assert.equal(rows[2].sameNativeEndpoint,true);
  assert.equal(rows[2].remote,'prflx');
  assert.equal(rows[4].state,'connected');
  assert(trace.nominated);
  for (const secret of ['secret','private','192.0.2','50000']) assert(!JSON.stringify(rows).includes(secret));
});

test('hold control defers browser candidates and ends while preserving restart credentials', () => {
  const held = [];
  const description = {description:{type:'offer',sdp:'v=0\r\na=ice-ufrag:fresh\r\na=candidate:example\r\na=end-of-candidates\r\n'}};
  const controlled = holdBrowserSignal(description,held);
  assert.equal(controlled.description.sdp,'v=0\r\na=ice-ufrag:fresh\r\n');
  assert(description.description.sdp.includes('a=candidate:'));
  const candidate = {candidate:{candidate:'candidate:example'},generation:'fresh'};
  const end = {candidate:null,generation:'fresh'};
  assert.equal(holdBrowserSignal(candidate,held),null);
  assert.equal(holdBrowserSignal(end,held),null);
  assert.deepEqual(held,[candidate,end]);
});

test('failure ring replay does not count as a fresh nomination', () => {
  const trace = new FirefoxPairOrder({address:'192.0.2.1',port:50000}, 'oldUfrag');
  trace.freshUfrag = 'newUfrag';
  trace.record('before-answer');
  const pair = '): nominated pair is private-pair|IP4:192.0.2.2:50001/UDP|IP4:192.0.2.1:50000/UDP(host|host)';
  trace.push('I/mtransport ICE-PEER(secret)/STREAM(transport (nested) - oldUfrag:private-password)/CAND-PAIR(private-pair)' + pair, []);
  assert.equal(trace.nominated,false);
  trace.push('D/nicer ICE-PEER(secret)/STREAM(transport (nested) - newUfrag:private-password)/CAND-PAIR(private-pair)' + pair, []);
  assert.equal(trace.nominated,true);
  const rows=trace.snapshot().events;
  assert.deepEqual(rows.slice(1).map(row=>[row.event,row.generation]),[
    ['replayed-nomination','previous'],['nominated','fresh']
  ]);
  assert(!JSON.stringify(rows).includes('Ufrag'));
  assert(!JSON.stringify(rows).includes('private-password'));
});

(async function () {
  globalThis.restartStage = 'initial negotiation';
  const a = new RTCPeerConnection(config), b = new RTCPeerConnection(config);
  const channels = [], received = [];
  let incoming, serial = 0, negotiations = 0;
  const ready = new Promise(resolve => incoming = resolve);
  const channel = a.createDataChannel('restart'); channels.push(channel);
  channel.binaryType = 'arraybuffer';
  b.ondatachannel = e => {
    channels.push(e.channel); e.channel.binaryType = 'arraybuffer';
    e.channel.onmessage = m => e.channel.send(m.data);
    incoming(e.channel);
  };
  channel.onmessage = e => received.push(e.data);
  a.onicecandidate = e => signal(b, e.candidate);
  b.onicecandidate = e => signal(a, e.candidate);
  a.onicecandidateerror = b.onicecandidateerror = () => { problem = 'gathering error'; };
  const queued = new Map([[a, []], [b, []]]);
  function ufrag(description) { return description?.sdp.match(/a=ice-ufrag:([^\r\n]+)/)?.[1]; }
  function pwd(description) { return description?.sdp.match(/a=ice-pwd:([^\r\n]+)/)?.[1]; }
  function signal(peer, candidate) {
    if (candidate && (!peer.remoteDescription || candidate.usernameFragment !== ufrag(peer.remoteDescription))) {
      queued.get(peer).push(candidate); return;
    }
    peer.addIceCandidate(candidate).catch(e => { problem = 'candidate:' + e.name; });
  }
  function flush(peer) { for (const candidate of queued.get(peer).splice(0)) signal(peer, candidate); }
  function check(value, message) { if (!value) throw new Error(message); }
  async function wait(predicate) {
    while (!predicate()) { if (problem) throw new Error(problem); await new Promise(r => setTimeout(r, 1)); }
  }
  async function ping() {
    const text = 'restart:' + (++serial);
    channel.send(text); await wait(() => received.includes(text));
    check(channels.every(c => c.readyState === 'open'), 'channel changed state');
  }
  async function exchange(left, right, offer) {
    await left.setLocalDescription(offer);
    await ping();
    await right.setRemoteDescription(left.localDescription); flush(right);
    await ping();
    await right.setLocalDescription();
    await ping();
    await left.setRemoteDescription(right.localDescription); flush(left);
    await ping();
  }
  // Closed restart is a no-op, including a second call.
  const closed = new RTCPeerConnection(config); closed.close(); closed.restartIce(); closed.restartIce();
  a.restartIce(); // No established/pending credentials: no extra negotiation.
  await a.setLocalDescription(); await b.setRemoteDescription(a.localDescription); flush(b);
  await b.setLocalDescription(); await a.setRemoteDescription(b.localDescription); flush(a);
  await ready; await wait(() => channel.readyState === 'open'); await ping();
  a.onnegotiationneeded = () => { negotiations++; };
  const old = a.localDescription, stale = a.sctp.transport.iceTransport.getLocalCandidates()[0].toJSON();
  stale.usernameFragment = ufrag(old);
  let synchronous = true;
  a.onnegotiationneeded = () => { check(!synchronous, 'synchronous negotiation event'); negotiations++; };
  a.restartIce(); a.restartIce(); synchronous = false;
  restartStage = 'restart event';
  await wait(() => negotiations === 1);
  const first = await a.createOffer();
  check(ufrag(first) !== ufrag(old) && pwd(first) !== pwd(old), 'credentials not refreshed');
  await ping(); // Creating/discarding an offer leaves the data path untouched.
  await a.setLocalDescription(first);
  await ping();
  await a.setLocalDescription({type:'rollback'});
  restartStage = 'rollback event';
  await wait(() => negotiations === 2);
  await ping();
  restartStage = 'post-rollback exchange';
  await exchange(a, b, await a.createOffer());
  check(ufrag(a.localDescription) !== ufrag(old), 'rollback lost restart request');
  try { await b.addIceCandidate(stale); throw new Error('stale generation accepted'); }
  catch (e) { check(e.name === 'OperationError', 'wrong stale-candidate exception'); }
  await ping();
  // The other offerer uses the explicit offer option, preserving the same channels.
  const otherOld = b.localDescription;
  restartStage = 'explicit offer option';
  const otherOffer = await b.createOffer({iceRestart:true});
  check(ufrag(otherOffer) !== ufrag(otherOld) && pwd(otherOffer) !== pwd(otherOld), 'offer option ignored');
  await exchange(b, a, otherOffer);
  // A request during a pending offer must survive that offer completing.
  await a.setLocalDescription(await a.createOffer({iceRestart:true}));
  restartStage = 'request during pending offer';
  a.restartIce();
  await b.setRemoteDescription(a.localDescription); flush(b);
  await b.setLocalDescription(); await a.setRemoteDescription(b.localDescription); flush(a);
  await wait(() => negotiations === 3);
  await exchange(a, b); // Automatic offer generation must also honor restartIce().
  restartStage = 'final ICE selection';
  await wait(() => a.iceConnectionState === 'connected' && b.iceConnectionState === 'connected');
  check(negotiations === 3 && channels.length === 2, 'duplicate event or replacement channel');
  for (const peer of [a, b]) {
    const report = await peer.getStats();
    for (const row of report.values()) if (row.type === 'candidate-pair') {
      check(report.has(row.localCandidateId) && report.has(row.remoteCandidateId), 'dangling restart stats reference');
    }
  }
  a.close(); b.close(); a.restartIce(); b.restartIce();
  done = true;
})().catch(e => { problem = e.name + ':' + e.message; });

// ICE-restart checks shared by the installed-browser receiving runner.
// Signaling descriptions stay in memory; receipts contain only assertions.
export async function runRestartChecks({native,browser,udpFixture,offerer,route,verifySnapshot,verifyStats}) {
  const peers={native,browser}, opposite=name=>name==='native'?'browser':'native';
  const assert=(value,message)=>{if(!value)throw Error(message);};
  const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
  const results=[];let probeRound=0,step='capture',snapshots,method,initiator;
  async function stats(peer) {
    await peer.evaluate('__interop.inspect()');
    for(let i=0;i<200;i++){const value=await peer.value('__interop.stats');if(value)return value;await delay(10);}
    throw Error('restart stats deadline');
  }
  try {
    for([method,initiator] of [['method',offerer],['offer',opposite(offerer)]]) {
      const descriptions=[];
      async function pump() {
        for(const name of ['native','browser'])for(const signal of await peers[name].value('__interop.takeSignals()')) {
          if(signal.description)descriptions.push({from:name,signal});
          else await peers[opposite(name)].evaluate(`__interop.receive(${JSON.stringify(signal)})`);
        }
        assert(!udpFixture.error,udpFixture.error);
      }
      async function wait(predicate,label) {
        step=label;const deadline=Date.now()+15000;
        while(Date.now()<deadline) {
          await pump();
          snapshots={native:await native.value('__interop.snapshot()'),browser:await browser.value('__interop.snapshot()')};
          assert(!snapshots.native.errors.length&&!snapshots.browser.errors.length,'restart peer error');
          if(predicate())return;
          await delay(10);
        }
        throw Error('restart '+label+' deadline');
      }
      async function probe(label,connected=false) {
        const round=++probeRound;
        for(const peer of Object.values(peers))await peer.evaluate(`__interop.beginRound(${round})`);
        for(const peer of Object.values(peers))await peer.evaluate('__interop.sendRound()');
        await wait(()=>Object.values(snapshots).every(s=>s.received.length===3),label);
        verifySnapshot(snapshots.native,'browser',connected);verifySnapshot(snapshots.browser,'native',connected);
        assert(Object.values(snapshots).every(s=>s.channelsPreserved&&s.channelCount===2&&s.channelStates.every(c=>c==='open')),'restart replaced or closed a channel');
        const observed=await stats(native);
        assert(observed.candidateReferencesValid&&observed.remoteAddressesRedacted,'restart stats graph or privacy failed during negotiation');
      }
      const before=await stats(native);
      for(const peer of Object.values(peers))await peer.evaluate('__interop.captureRestart()');
      await peers[initiator].evaluate(`__interop.restart(${JSON.stringify(method)})`);
      await wait(()=>descriptions.some(s=>s.signal.description.type==='offer'),'local offer');
      const offer=descriptions.shift();assert(offer.from===initiator&&offer.signal.description.type==='offer','unexpected restart offer');
      await probe('after local offer');
      const answerer=opposite(initiator);
      await peers[answerer].evaluate(`__interop.receive(${JSON.stringify(offer.signal)})`);
      await wait(()=>snapshots[answerer].waitingToAnswer,'remote offer');
      await probe('after remote offer');
      await peers[answerer].evaluate('__interop.answer()');
      await wait(()=>descriptions.some(s=>s.signal.description.type==='answer'),'local answer');
      const answer=descriptions.shift();assert(answer.from===answerer&&answer.signal.description.type==='answer','unexpected restart answer');
      await probe('after local answer');
      await peers[initiator].evaluate(`__interop.receive(${JSON.stringify(answer.signal)})`);
      await wait(()=>Object.values(snapshots).every(s=>s.signaling==='stable'&&s.localCredentialsChanged&&s.remoteCredentialsChanged&&['connected','completed'].includes(s.iceState)&&s.gathering==='complete'),'fresh ICE connected');
      await probe('after remote answer',true);
      await native.evaluate('__interop.rejectStale()');
      await wait(()=>snapshots.native.staleRefused,'stale candidate refusal');
      const a=await stats(native),b=await stats(browser);verifyStats(a,b,route);
      assert(a.localFingerprint===before.localFingerprint&&a.remoteFingerprint===before.remoteFingerprint,'restart changed certificate identity');
      const expectedEvents=method==='method'&&initiator==='native'?1:0;
      assert(snapshots.native.restartNegotiations===expectedEvents,'restart negotiationneeded count mismatch');
      const relayAllocationReused=route!=='relay'||(a.localEndpoint!==null&&a.localEndpoint===before.localEndpoint);
      assert(relayAllocationReused,'native TURN allocation replaced during restart');
      results.push({method,initiator,credentialsChangedOnBothPeers:true,channelIdentityPreserved:true,certificateIdentityPreserved:true,nativeNegotiationEvents:expectedEvents,liveChannelProbes:4,binaryBytesPerPeer:192000,staleCandidateRefused:true,relayAllocationReused});
      assert(!descriptions.length,'unexpected duplicate negotiation');
    }
    return results;
  } catch(error) {
    error.restartDiagnostics={step,method,initiator,snapshots};throw error;
  }
}

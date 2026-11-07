// ICE-restart checks shared by the installed-browser receiving runner.
// Signaling descriptions stay in memory; receipts contain only assertions.
import {watchFirefoxClose} from './webrtc_firefox_close_trace.mjs';
import {RestartSignals} from './webrtc_restart_signals.mjs';
import {FirefoxPairOrder, fixtureAnswers, holdBrowserSignal} from './webrtc_firefox_pair_order.mjs';
export async function runRestartChecks({native,browser,udpFixture,offerer,route,verifySnapshot,verifyStats,answerDelayMs=0,mapBrowserSignal=signal=>signal,requireMdns=false,verifyAdvertisement=null,restartDiagnostic=null,localBind=null,mdnsFixture=null}) {
  const peers={native,browser}, opposite=name=>name==='native'?'browser':'native';
  const assert=(value,message)=>{if(!value)throw Error(message);};
  const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
  const results=[];let probeRound=0,step='capture',snapshots,method,initiator,iceBefore,statsObserved,signalEvidence;
  const traceOrder=restartDiagnostic?.startsWith('pair-order');
  let pairOrder, answersPending, fixtureAnswerCount, browserCloseTrace;
  const orderSummary=()=>pairOrder?{...pairOrder.snapshot(),fixtureAnswerCount,browserCloseTrace}:undefined;
  async function stats(peer) {
    await peer.evaluate('__interop.inspect()');
    for(let i=0;i<200;i++){const value=await peer.value('__interop.stats');if(value)return value;await delay(10);}
    throw Error('restart stats deadline');
  }
  const nativeHost=()=>native.value('__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type==="host").map(c=>({address:c.address,port:c.port}))[0]');
  async function hostEvidence() {
    if(!signalEvidence)return;
    await stats(browser);
    const host=await nativeHost();
    if(host)signalEvidence.record({type:'remote-hosts',view:await browser.value(`__interop.hostEvidence(${JSON.stringify(host.address)},${JSON.stringify(localBind)},${host.port})`)});
  }
  try {
    for([method,initiator] of [['method',offerer],['offer',opposite(offerer)]]) {
      signalEvidence=restartDiagnostic?new RestartSignals(restartDiagnostic):null;
      const metricsBefore=await native.request('metrics');
      iceBefore=metricsBefore.ice;
      const descriptions=[];
      const held=[];
      let holding=restartDiagnostic?.startsWith('pair-order-hold')&&initiator==='browser';
      if(traceOrder) {
        const host=await nativeHost();
        const previous=initiator==='native'?await browser.value(String.raw`__interop.pc.localDescription.sdp.match(/^a=ice-ufrag:([^\r\n]+)/m)?.[1]`):null;
        pairOrder=new FirefoxPairOrder(host?{address:localBind,port:host.port}:null,previous);
        if(previous)await browser.evaluate(`(${watchFirefoxClose.toString()})()`);
        browserCloseTrace=undefined;
        browser.firefoxTrace.pairOrder=pairOrder;
        fixtureAnswerCount=0;
      }
      async function pump() {
        for(const name of ['native','browser'])for(let signal of await peers[name].value('__interop.takeSignals()')) {
          if(name==='browser')signal=await mapBrowserSignal(signal);
          if(signalEvidence)signal=signalEvidence.map(name,signal,step);
          if(signal&&holding&&name==='browser')signal=holdBrowserSignal(signal,held);
          if(!signal)continue;
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
          if(predicate()){if(signalEvidence)signalEvidence.record({type:'phase',step,firefox:browser.firefoxTrace?.snapshot()});return;}
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
      if(initiator==='native'&&restartDiagnostic==='pair-order-hold-assist-native') {
        answersPending=fixtureAnswers(offer.signal,mdnsFixture,localBind)
          .then(count=>{fixtureAnswerCount=count;},()=>{fixtureAnswerCount=-1;});
      }
      const localAnswer = pairOrder?.previousUfrag ? '__interop.closeTraceMark("answer-requested");__interop.answer()' : '__interop.answer()';
      await peers[answerer].evaluate(localAnswer);
      await wait(()=>descriptions.some(s=>s.signal.description.type==='answer'),'local answer');
      const answer=descriptions.shift();assert(answer.from===answerer&&answer.signal.description.type==='answer','unexpected restart answer');
      if(pairOrder?.previousUfrag) {
        pairOrder.freshUfrag=answer.signal.description.sdp.match(/^a=ice-ufrag:([^\r\n]+)/m)?.[1];
        pairOrder.record('browser-answer-available');
        await browser.evaluate('__interop.closeTraceMark("answer-available")');
      }
      await probe('after local answer');
      if(answerDelayMs)await delay(answerDelayMs);
      if(pairOrder) {
        const ice=(await native.request('metrics')).ice;
        pairOrder.record('before-answer',{
          nativeChecksSent:ice.checksSent-iceBefore.checksSent,
          nativeNominationsSent:ice.nominationsSent-iceBefore.nominationsSent,
          held:held.length
        });
        if(initiator==='browser') {
          answersPending=fixtureAnswers(answer.signal,mdnsFixture,localBind)
            .then(count=>{fixtureAnswerCount=count;},()=>{fixtureAnswerCount=-1;});
        }
      }
      await peers[initiator].evaluate(`__interop.receive(${JSON.stringify(answer.signal)})`);
      pairOrder?.record('answer-dispatched');
      if(holding) {
        await wait(()=>pairOrder.nominated,'candidate nomination with browser candidates held');
        const observed=await stats(browser);
        pairOrder.record('release-browser-candidates',{count:held.length,remote:observed.pair?.remote});
        holding=false;
        for(const signal of held.splice(0))await native.evaluate(`__interop.receive(${JSON.stringify(signal)})`);
      }
      await wait(()=>Object.values(snapshots).every(s=>s.signaling==='stable'&&s.localCredentialsChanged&&s.remoteCredentialsChanged&&['connected','completed'].includes(s.iceState)&&s.gathering==='complete'),'fresh ICE connected');
      if(signalEvidence)for(const signal of signalEvidence.flush())await browser.evaluate(`__interop.receive(${JSON.stringify(signal)})`);
      await probe('after remote answer',true);
      await native.evaluate('__interop.rejectStale()');
      await wait(()=>snapshots.native.staleRefused,'stale candidate refusal');
      let a,b,attempts=0;statsObserved=[];
      const statsDeadline=Date.now()+15000;
      while(Date.now()<statsDeadline) {
        a=await stats(native);b=await stats(browser);attempts++;
        const pairStates=[a.pair?.state||null,b.pair?.state||null];
        if(statsObserved.length<8&&JSON.stringify(pairStates)!==JSON.stringify(statsObserved.at(-1)))statsObserved.push(pairStates);
        assert(a.candidateReferencesValid&&a.remoteAddressesRedacted,'restart stats graph or privacy failed');
        if(pairStates.every(state=>state==='succeeded'))break;
        await delay(25);
      }
      verifyStats(a,b,route);
      if(answersPending) {
        await answersPending;
        answersPending=null;
      }
      if(fixtureAnswerCount===-1)throw Error('fixture answer control failed');
      pairOrder?.record('selected-route',{remote:b.pair?.remote});
      if(pairOrder?.previousUfrag)browserCloseTrace=await browser.value('__interop.closeTrace()');
      await hostEvidence();
      const advertisement=verifyAdvertisement?await verifyAdvertisement(a,b):undefined;
      let mdnsResolved;
      if(requireMdns&&route!=='relay'){
        const deadline=Date.now()+6000;
        do{mdnsResolved=(await native.request('metrics')).mdnsResolved-metricsBefore.mdnsResolved;if(mdnsResolved>=1)break;await delay(25);}while(Date.now()<deadline);
        assert(mdnsResolved>=1,'fresh restart lacked successful mDNS resolution');
      }
      assert(a.localFingerprint===before.localFingerprint&&a.remoteFingerprint===before.remoteFingerprint,'restart changed certificate identity');
      const expectedEvents=method==='method'&&initiator==='native'?1:0;
      assert(snapshots.native.restartNegotiations===expectedEvents,'restart negotiationneeded count mismatch');
      const relayAllocationReused=route!=='relay'||(a.localEndpoint!==null&&a.localEndpoint===before.localEndpoint);
      assert(relayAllocationReused,'native TURN allocation replaced during restart');
      results.push({method,initiator,mdnsResolved,advertisement,pairOrder:orderSummary(),signalDiagnostics:signalEvidence?{...signalEvidence.summary(),firefox:browser.firefoxTrace?.snapshot()}:undefined,credentialsChangedOnBothPeers:true,channelIdentityPreserved:true,certificateIdentityPreserved:true,nativeNegotiationEvents:expectedEvents,liveChannelProbes:4,binaryBytesPerPeer:192000,staleCandidateRefused:true,relayAllocationReused,nativeIce:{before:iceBefore,after:(await native.request('metrics')).ice},statsSettling:attempts>1?{attempts,pairStates:statsObserved}:undefined});
      assert(!descriptions.length,'unexpected duplicate negotiation');
    }
    return results;
  } catch(error) {
    if(answersPending)await answersPending;
    if(pairOrder?.previousUfrag)try{browserCloseTrace=await browser.value('__interop.closeTrace()');}catch{}
    try{await hostEvidence();}catch{}
    error.restartDiagnostics={pairOrder:orderSummary(),completedPairOrders:traceOrder?results.map(r=>r.pairOrder):undefined,signalDiagnostics:signalEvidence?{...signalEvidence.summary(),firefox:browser.firefoxTrace?.snapshot()}:undefined,completedRestarts:results.length,step,method,initiator,snapshots,iceBefore,iceAfter:(await native.request('metrics')).ice,statsObserved};throw error;
  }
}

// A connected peer can publish its selected-pair statistics asynchronously.
// Resample within a deadline; route, certificate and privacy assertions remain
// the caller's acceptance checks after both peers report success.
export async function settledStats(native,browser,{timeoutMs=15000,pollMs=25}={}) {
  const deadline=Date.now()+timeoutMs,states=[];
  const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
  let a,b,attempts=0;
  async function sample(peer) {
    await peer.evaluate('__interop.inspect()');
    while(Date.now()<deadline) {
      const value=await peer.value('__interop.stats');
      if(value)return value;
      await delay(pollMs);
    }
    return null;
  }
  while(Date.now()<deadline) {
    a=await sample(native);b=await sample(browser);attempts++;
    const pairStates=[a?.pair?.state||null,b?.pair?.state||null];
    if(states.length<8&&JSON.stringify(pairStates)!==JSON.stringify(states.at(-1)))states.push(pairStates);
    if(a&&(!a.candidateReferencesValid||!a.remoteAddressesRedacted))throw Error('native stats graph or privacy failed');
    if(pairStates.every(state=>state==='succeeded'))return {a,b,settling:attempts>1?{attempts,pairStates:states}:undefined};
    await delay(pollMs);
  }
  const error=Error('selected candidate pair statistics deadline');
  error.initialStatsDiagnostics={attempts,pairStates:states};throw error;
}

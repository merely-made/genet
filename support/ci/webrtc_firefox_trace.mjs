// Reduce process-local Firefox diagnostic logs to fixed counters. Never retain
// or emit log text, SDP, addresses, hostnames, credentials or payloads.
const patterns={
  mdnsQueries:/\bQuerying [^ ]+\.local\b/,
  mdnsResponses:/mDNS response:/,
  mdnsSendFailures:/Sending mDNS query failed/,
  mdnsSocketFailures:/Could not start mDNS Service|Socket error:|Could not join multicast group/,
  mdnsParseFailures:/Could not parse mDNS packet/,
  remoteCandidateCalls:/AddIceCandidate:/,
  remoteNamedCandidateCalls:/AddIceCandidate:.*\.local/,
  candidateParseFailures:/Failed to (?:parse trickle candidate|incorporate remote candidate)/i,
  noCandidatePairs:/no (?:candidate )?pairs|no (?:remote )?candidates|empty check ?list/i,
  iceFailures:/ICE.*fail|stream.*fail|checks.*fail/i,
  iceStateChanges:/ICE.*state|state.*ICE/i,
  consentFailures:/consent.*(?:fail|expire)/i,
};
const labels=new Set('ice candidate candidates remote local stream failed failure fail error empty no checks checklist trickle component parse invalid mdns query resolve resolved timeout waiting pending consent address socket'.split(' '));
export class FirefoxTrace {
  constructor(){this.partial='';this.clear();}
  clear(){this.errors=[];this.counts={lines:0,oversized:0,...Object.fromEntries(Object.keys(patterns).map(k=>[k,0]))};}
  push(chunk){
    const text=this.partial+chunk.toString();this.partial='';
    const lines=text.split(/\r?\n/);const last=lines.pop();
    this.partial=last.length<=4096?last:'';if(last.length>4096)this.counts.oversized++;
    for(const line of lines){
      if(line.length>4096){this.counts.oversized++;continue;}
      this.counts.lines++;
      for(const [key,pattern] of Object.entries(patterns))if(pattern.test(line))this.counts[key]++;
      if(/error|fail|could not|no candidates/i.test(line)){
        const words=[...new Set(line.toLowerCase().split(/[^a-z]+/).filter(w=>labels.has(w)))];
        if(words.length){this.errors.push(words);if(this.errors.length>16)this.errors.shift();}
      }
    }
  }
  snapshot(){return {...this.counts,errorLabels:this.errors.map(words=>[...words])};}
}

// Reduce process-local Firefox diagnostic logs to fixed counters. Never retain
// or emit log text, SDP, addresses, hostnames, credentials or payloads.
const patterns={
  mdnsQueries:/\bQuerying [^ ]+\.local\b/,
  mdnsResponses:/mDNS response:/,
  mdnsSendFailures:/Sending mDNS query failed/,
  mdnsSocketFailures:/Could not start mDNS Service|Socket error:|Could not join multicast group/,
  mdnsParseFailures:/Could not parse mDNS packet/,
  remoteCandidateCalls:/AddIceCandidate:/,
  transportParseCalls:/parsing trickle candidate/,
  transportProcessFailures:/Couldn't process ICE candidate/,
  transportMissingStreams:/No ICE stream for candidate/,
  transportRedundant:/Trickle candidate is redundant/,
  transportUnusedComponent:/Trickle candidate is ignored/,
  remoteNamedCandidateCalls:/AddIceCandidate:.*\.local/,
  candidateParseFailures:/Failed to (?:parse trickle candidate|incorporate remote candidate)/i,
  noCandidatePairs:/no (?:candidate )?pairs|no (?:remote )?candidates|empty check ?list/i,
  iceFailures:/ICE.*fail|stream.*fail|checks.*fail/i,
  iceStateChanges:/ICE.*state|state.*ICE/i,
  consentFailures:/consent.*(?:fail|expire)/i,
};
const labels=new Set('ice candidate candidates remote local stream failed failure fail error empty no checks checklist trickle component parse invalid mdns query resolve resolving resolved timeout waiting pending consent address socket dns resolver cache cached hit miss positive negative unknown host trr complete completed getaddrinfo bypass success cancel canceled offline unavailable flags ipv4 ipv6'.split(' '));
export class FirefoxTrace {
  constructor(){this.partial='';this.stdoutPartial='';this.clear();}
  clear(){this.partial='';this.stdoutPartial='';this.timecards=[];this.timecardEvents=[];this.timecardOmitted=0;this.transportEvents=[];this.transportOmitted=0;this.names=new Set();this.dns=[];this.errors=[];this.counts={lines:0,oversized:0,nativeDnsLines:0,...Object.fromEntries(Object.keys(patterns).map(k=>[k,0]))};}
  watch(names){for(const name of names){if(this.names.size>=128&&!this.names.has(name))throw Error('Firefox name-observation budget');this.names.add(name);}}
  push(chunk){
    const text=this.partial+chunk.toString();this.partial='';
    const lines=text.split(/\r?\n/);const last=lines.pop();
    this.partial=last.length<=4096?last:'';if(last.length>4096)this.counts.oversized++;
    for(const line of lines){
      if(line.length>4096){this.counts.oversized++;continue;}
      this.counts.lines++;
      this.pairOrder?.push(line, this.names);
      if([...this.names].some(name=>line.includes(name))){
        const event=/parsing trickle candidate/.test(line)?'parse':/Couldn't process ICE candidate/.test(line)?'process-failed':/No ICE stream for candidate/.test(line)?'missing-stream':/Trickle candidate is redundant/.test(line)?'redundant':/Trickle candidate is ignored/.test(line)?'unused-component':null;
        if(event){if(this.transportEvents.length<64)this.transportEvents.push(event);else this.transportOmitted++;}
      }
      for(const [key,pattern] of Object.entries(patterns))if(pattern.test(line))this.counts[key]++;
      if(/nsHostResolver|nsDNSService/.test(line)&&[...this.names].some(name=>line.includes(name))){
        this.counts.nativeDnsLines++;
        this.dns.push([...new Set(line.toLowerCase().split(/[^a-z]+/).filter(w=>labels.has(w)))]);
        if(this.dns.length>16)this.dns.shift();
      }
      if(/error|fail|could not|no candidates/i.test(line)){
        const words=[...new Set(line.toLowerCase().split(/[^a-z]+/).filter(w=>labels.has(w)))];
        if(words.length){this.errors.push(words);if(this.errors.length>16)this.errors.shift();}
      }
    }
  }
  pushStdout(chunk){
    const lines=(this.stdoutPartial+chunk.toString()).split(/\r?\n/);const last=lines.pop();
    this.stdoutPartial=last.length<=4096?last:'';
    for(const line of lines){
      if(line.length>4096)continue;
      if(/^Timecard created \d+\.\d+/.test(line)){if(this.timecards.length<8)this.timecards.push(this.timecardEvents.length);continue;}
      const match=line.match(/^\s*(\d+)\.(\d{6})\s*\|\s*\d+\.\d{6}\s*\|\s*(.*?)\s*\|/);
      if(!match)continue;
      const events={'Look up mDNS name':'lookup','Done looking up mDNS name':'resolved-forwarded','Failed looking up mDNS name':'lookup-failed','Set Remote Description':'remote-description','Set Local Description':'local-description','Add Ice Candidate':'candidate-api','Create Offer':'create-offer','Create Answer':'create-answer','Close':'close','Destructor Invoked':'destroy'};
      const event=events[match[3]];if(!event)continue;
      if(this.timecardEvents.length<128)this.timecardEvents.push({event,ms:Number(match[1])*1000+Math.floor(Number(match[2])/1000)});else this.timecardOmitted++;
    }
  }
  snapshot(){return {...this.counts,timecards:this.timecards.length,timecardEvents:this.timecardEvents.map(x=>({...x})),timecardOmitted:this.timecardOmitted,transportEvents:[...this.transportEvents],transportOmitted:this.transportOmitted,dnsLabels:this.dns.map(words=>[...words]),errorLabels:this.errors.map(words=>[...words])};}
}

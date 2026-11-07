// Bounded, credential-free signaling evidence and explicit diagnostic controls.
const endCandidate=s=>'candidate' in s&&(!s.candidate||s.candidate.candidate==='');
export function summarizeSignal(signal) {
  if(signal.description){
    const lines=signal.description.sdp.split(/\r?\n/);
    const candidates=lines.filter(s=>s.startsWith('a=candidate:')).map(s=>s.split(/\s+/));
    return {type:signal.description.type,candidates:candidates.length,
      namedHosts:candidates.filter(f=>f[7]==='host'&&f[4].endsWith('.local')).length,
      numericHosts:candidates.filter(f=>f[7]==='host'&&!f[4].endsWith('.local')).length,
      components:[...new Set(candidates.map(f=>Number(f[1])))],
      ends:lines.filter(s=>s==='a=end-of-candidates').length,
      trickle:lines.some(s=>s.startsWith('a=ice-options:')&&s.split(/\s|:/).includes('trickle'))};
  }
  if(endCandidate(signal))return {type:'end'};
  const f=signal.candidate.candidate.split(/\s+/);
  return {type:'candidate',component:Number(f[1]),candidateType:f[7],named:f[4].endsWith('.local'),
    generationMatches:!signal.candidate.usernameFragment||signal.candidate.usernameFragment===signal.generation};
}
export class RestartSignals {
  constructor(mode){this.mode=mode;this.entries=[];this.omitted=0;this.ends=[];}
  record(value){if(this.entries.length<64)this.entries.push(value);else this.omitted++;}
  map(from,signal,step){
    const entry={from,step,...summarizeSignal(signal)};this.record(entry);
    if(from!=='native'||this.mode==='observe'||this.mode.startsWith('pair-order'))return signal;
    if(this.mode==='trickle-option'){
      if(!signal.description||entry.trickle)return signal;
      const lines=signal.description.sdp.split(/\r?\n/),media=lines.findIndex(s=>s.startsWith('m='));
      const at=lines.findIndex((s,i)=>(media<0||i<media)&&s.startsWith('a=ice-options:'));
      if(at>=0)lines[at]+=' trickle';else lines.splice(media<0?0:media,0,'a=ice-options:trickle');
      entry.trickleAdded=true;
      return {...signal,description:{...signal.description,sdp:lines.join('\r\n')}};
    }
    if(endCandidate(signal)){
      if(this.ends.length>=8)throw Error('restart diagnostic end-marker budget');
      this.ends.push(signal);entry.held=true;return null;
    }
    if(!signal.description)return signal;
    const sdp=signal.description.sdp.replace(/^a=end-of-candidates\r?\n/gm,'');
    entry.embeddedEndsDeferred=entry.ends;
    return {...signal,description:{...signal.description,sdp:this.mode==='trickle'?sdp.replace(/^a=candidate:.*\r?\n/gm,''):sdp}};
  }
  flush(){const ends=this.ends.splice(0);this.record({type:'flush',count:ends.length});return ends;}
  summary(){return {mode:this.mode,entries:this.entries,omitted:this.omitted,pendingEnds:this.ends.length};}
}

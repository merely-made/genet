// Controlled IPv4 one-shot responder. Other browser mDNS names stay untouched.
import { observeNames } from './webrtc_mdns_observer.mjs';
import { createSocket } from 'node:dgram';
import { randomUUID } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const execFileAsync=promisify(execFile);

export class MdnsFixture {
  constructor(){this.nativeQuerySources={selected:0,additional:0,other:0};this.sharedQueries={selected:0,additional:0,other:0};this.nativeNames=new Set();this.nativeQueries=0;this.nativeAnswers=0;this.names=new Map();this.endpoints=new Map();this.browserNames=new Set();this.aliasedBrowserNames=new Set();this.ipv4Ports=new Set();this.seenPorts=new Set();this.queries=0;this.answers=0;this.held=0;this.error=null;this.pending=0;}
  async prepareBrowser(signal,pids) {
    const generation=signal.generation||signal.description?.sdp.match(/a=ice-ufrag:([^\r\n]+)/)?.[1];
    if(generation&&generation!==this.browserGeneration){this.browserGeneration=generation;this.ipv4Ports.clear();this.seenPorts.clear();}
    const candidates=signal.candidate?[signal.candidate.candidate]:[...(signal.description?.sdp||'').matchAll(/^a=(candidate:[^\r\n]+)/gm)].map(m=>m[1]);
    if(!candidates.some(c=>{const f=c.split(/\s+/);return f[4]?.toLowerCase().endsWith('.local')&&!this.seenPorts.has(Number(f[5]));}))return;
    if(!pids.every(p=>Number.isSafeInteger(p)&&p>0))throw Error('invalid browser socket owner');
    const script=`$owners=[Collections.Generic.HashSet[int]]::new();${pids.map(p=>`[void]$owners.Add(${p});`).join('')}$rows=@(Get-CimInstance Win32_Process -Filter "Name='firefox.exe'");do{$changed=$false;foreach($r in $rows){if($owners.Contains([int]$r.ParentProcessId)){$changed=$owners.Add([int]$r.ProcessId)-or $changed}}}while($changed);@(Get-NetUDPEndpoint -ErrorAction SilentlyContinue | Where-Object {$owners.Contains([int]$_.OwningProcess)} | ForEach-Object {@{port=$_.LocalPort;ipv4=($_.LocalAddress -eq '0.0.0.0' -or $_.LocalAddress -eq '${this.bind}')}}) | ConvertTo-Json -Compress`;
    const {stdout}=await execFileAsync('C:/Program Files/PowerShell/7/pwsh.exe',['-NoProfile','-NonInteractive','-Command',script],{encoding:'utf8',windowsHide:true,timeout:15000,maxBuffer:65536});
    const parsed=JSON.parse(stdout.trim()||'[]');
    for(const row of Array.isArray(parsed)?parsed:[parsed]){this.seenPorts.add(row.port);if(row.ipv4)this.ipv4Ports.add(row.port);}
  }
  async start(bind,extraInterface=null) {
    this.extraInterface=extraInterface;
    this.bind=bind;this.socket=createSocket({type:'udp4',reuseAddr:true});
    this.socket.on('error',()=>{this.error='controlled mDNS socket failed';});
    this.socket.on('message',(packet,from)=>{
      const seen=observeNames(packet,this.nativeNames,this.bind);this.nativeQueries+=seen.queries;
      const source=from.address===this.bind?'selected':from.address===this.extraInterface?'additional':'other';
      this.nativeQuerySources[source]+=seen.queries;
      if(from.port===5353&&packet.length>=12&&!(packet.readUInt16BE(2)&0x8000))this.sharedQueries[source]++;
      if(from.port===5353&&from.address===this.bind)this.nativeAnswers+=seen.answers;
      if(packet.length<17||packet.length>512||packet.readUInt16BE(4)!==1||(packet.readUInt16BE(2)&0x8000))return;
      let end=12;const labels=[];
      while(end<packet.length&&packet[end]){const size=packet[end++];if(size>63||end+size>=packet.length)return;labels.push(packet.subarray(end,end+size).toString('ascii'));end+=size;}
      end++;const key=labels.join('.').toLowerCase();const entry=this.names.get(key);
      if(!entry||end+4!==packet.length||packet.readUInt16BE(end)!==1||packet.readUInt16BE(end+2)!==1)return;
      this.queries++;if(entry.hold){this.held++;return;}
      if(this.pending>=64){this.error='mDNS response queue budget exceeded';return;}
      const header=Buffer.alloc(12);header.writeUInt16BE(packet.readUInt16BE(0),0);header.writeUInt16BE(0x8400,2);header.writeUInt16BE(1,4);header.writeUInt16BE(1,6);
      const record=Buffer.from([0xc0,0x0c,0,1,0,1,0,0,0,120,0,4,...entry.ip.split('.').map(Number)]);
      this.pending++;this.socket.send(Buffer.concat([header,packet.subarray(12),record]),from.port,from.address,error=>{this.pending--;if(error)this.error='mDNS response failed';else this.answers++;});
    });
    await new Promise((resolve,reject)=>{this.socket.once('error',reject);this.socket.bind(5353,'0.0.0.0',resolve);});
    this.socket.addMembership('224.0.0.251',bind);
    if(extraInterface&&extraInterface!==bind)this.socket.addMembership('224.0.0.251',extraInterface);
  }
  watchNative(signal){
    const values=signal.candidate?[signal.candidate.candidate]:[...(signal.description?.sdp||'').matchAll(/^a=(candidate:[^\r\n]+)/gm)].map(m=>m[1]);
    for(const value of values){const f=value.split(/\s+/);if(f[7]==='host'&&f[4].endsWith('.local')){if(this.nativeNames.size>=128&&!this.nativeNames.has(f[4]))throw Error('native observation budget');this.nativeNames.add(f[4]);}}
  }
  heldName(){const name=randomUUID()+'.local';this.names.set(name,{ip:this.bind,hold:true});return name;}
  candidate(value,aliasBrowserNames=false) {
    const fields=value.split(/\s+/);
    if(fields[2]?.toLowerCase()!=='udp'||fields[6]!=='typ'||fields[7]!=='host')return value;
    const browserName=fields[4]?.toLowerCase().endsWith('.local');
    if(browserName&&(!aliasBrowserNames||!this.ipv4Ports.has(Number(fields[5])))){this.browserNames.add(fields[4].toLowerCase());return value;}
    // This opt-in receiving fixture maps local browser sockets to its selected
    // IPv4 host. It does not qualify that browser's own name advertisements.
    if(browserName)this.aliasedBrowserNames.add(fields[4].toLowerCase());
    else if(fields[4]!==this.bind)return value;
    const key=fields[4]+':'+fields[5];let name=this.endpoints.get(key);
    if(!name){if(this.names.size>=128)throw Error('mDNS fixture name budget exceeded');name=randomUUID()+'.local';this.endpoints.set(key,name);this.names.set(name,{ip:this.bind,hold:false});}
    fields[4]=name;return fields.join(' ');
  }
  rewrite(signal,aliasBrowserNames=false) {
    if(signal.candidate)return {...signal,candidate:{...signal.candidate,candidate:this.candidate(signal.candidate.candidate,aliasBrowserNames)}};
    if(signal.description)return {...signal,description:{...signal.description,sdp:signal.description.sdp.replace(/^a=(candidate:[^\r\n]+)/gm,(_,value)=>'a='+this.candidate(value,aliasBrowserNames))}};
    return signal;
  }
  snapshot(){return {observationInterfaces:1+Number(!!this.extraInterface&&this.extraInterface!==this.bind),nativeQuerySources:{...this.nativeQuerySources},sharedQueries:{...this.sharedQueries},nativeQueries:this.nativeQueries,nativeAnswers:this.nativeAnswers,queries:this.queries,answers:this.answers,held:this.held,controlledNames:this.endpoints.size,browserNames:this.browserNames.size,aliasedBrowserNames:this.aliasedBrowserNames.size};}
  clear(){this.nativeQuerySources={selected:0,additional:0,other:0};this.sharedQueries={selected:0,additional:0,other:0};this.nativeNames.clear();this.nativeQueries=0;this.nativeAnswers=0;this.names.clear();this.endpoints.clear();this.browserNames.clear();this.aliasedBrowserNames.clear();this.ipv4Ports.clear();this.seenPorts.clear();this.browserGeneration=null;}
  async close(){await new Promise(resolve=>this.socket.close(resolve));}
}

// Controlled IPv4 one-shot responder. Other browser mDNS names stay untouched.
import { createSocket } from 'node:dgram';
import { randomUUID } from 'node:crypto';

export class MdnsFixture {
  constructor(){this.names=new Map();this.endpoints=new Map();this.browserNames=new Set();this.queries=0;this.answers=0;this.held=0;this.error=null;this.pending=0;}
  async start(bind) {
    this.bind=bind;this.socket=createSocket({type:'udp4',reuseAddr:true});
    this.socket.on('error',()=>{this.error='controlled mDNS socket failed';});
    this.socket.on('message',(packet,from)=>{
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
  }
  heldName(){const name=randomUUID()+'.local';this.names.set(name,{ip:this.bind,hold:true});return name;}
  candidate(value) {
    const fields=value.split(/\s+/);
    if(fields[2]?.toLowerCase()!=='udp'||fields[6]!=='typ'||fields[7]!=='host')return value;
    if(fields[4]?.toLowerCase().endsWith('.local')){this.browserNames.add(fields[4].toLowerCase());return value;}
    if(fields[4]!==this.bind)return value;
    const key=fields[4]+':'+fields[5];let name=this.endpoints.get(key);
    if(!name){if(this.names.size>=128)throw Error('mDNS fixture name budget exceeded');name=randomUUID()+'.local';this.endpoints.set(key,name);this.names.set(name,{ip:this.bind,hold:false});}
    fields[4]=name;return fields.join(' ');
  }
  rewrite(signal) {
    if(signal.candidate)return {...signal,candidate:{...signal.candidate,candidate:this.candidate(signal.candidate.candidate)}};
    if(signal.description)return {...signal,description:{...signal.description,sdp:signal.description.sdp.replace(/^a=(candidate:[^\r\n]+)/gm,(_,value)=>'a='+this.candidate(value))}};
    return signal;
  }
  snapshot(){return {queries:this.queries,answers:this.answers,held:this.held,controlledNames:this.endpoints.size,browserNames:this.browserNames.size};}
  clear(){this.names.clear();this.endpoints.clear();this.browserNames.clear();}
  async close(){await new Promise(resolve=>this.socket.close(resolve));}
}

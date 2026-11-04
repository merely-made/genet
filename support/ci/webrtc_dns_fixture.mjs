// Controlled DNS for ICE-server qualification. Never changes OS DNS or hosts.
import { createSocket } from 'node:dgram';

const name=value=>Buffer.concat([...value.split('.').filter(Boolean).map(label=>Buffer.concat([Buffer.from([label.length]),Buffer.from(label)])),Buffer.from([0])]);
const text=value=>Buffer.concat([Buffer.from([value.length]),Buffer.from(value)]);

export class DnsFixture {
  constructor() { this.queries=0;this.answers=0;this.held=0;this.pause=false;this.pending=0;this.error=null;this.types={a:0,srv:0,naptr:0}; }
  async start(address,servicePort=null) {
    const octets=address.split('.').map(Number);
    if(octets.length!==4||octets.some(n=>!Number.isInteger(n)||n<0||n>255))throw Error('DNS fixture requires IPv4');
    if(servicePort!==null&&(!Number.isInteger(servicePort)||servicePort<1||servicePort>65535))throw Error('DNS fixture requires a valid service port');
    this.socket=createSocket('udp4');
    this.socket.on('error',()=>{this.error='controlled DNS socket failed';});
    this.socket.on('message',(packet,from)=>{
      if(from.address!=='127.0.0.1'||packet.length<17||packet.length>512||packet.readUInt16BE(4)!==1)return;
      const labels=[];let end=12;
      while(end<packet.length&&packet[end]) {
        const length=packet[end++];if(length>63||end+length>=packet.length)return;
        labels.push(packet.subarray(end,end+length).toString('ascii'));end+=length;
      }
      end++;
      const host=labels.join('.').toLowerCase();
      const allowed=servicePort===null?['rtc.example.test']:['rtc.example.test','_stun._udp.rtc.example.test','_turn._udp.rtc.example.test','_turn._udp.hosted.example.test','server.example.test'];
      if(end+4!==packet.length||!allowed.includes(host)||packet.readUInt16BE(end+2)!==1)return;
      const type=packet.readUInt16BE(end),kind={1:'a',33:'srv',35:'naptr'}[type];
      this.queries++;
      if(kind)this.types[kind]++;
      if(this.pause){this.held++;return;}
      if(this.pending>=64){this.error='controlled DNS reply budget exceeded';return;}
      const header=Buffer.alloc(12);let data=null;
      if(type===1&&(host==='rtc.example.test'||host==='server.example.test'))data=Buffer.from(octets);
      if(servicePort!==null&&type===33&&host.startsWith('_')) {
        const fields=Buffer.alloc(6);fields.writeUInt16BE(servicePort,4);
        data=Buffer.concat([fields,name('server.example.test')]);
      }
      if(servicePort!==null&&type===35&&host==='rtc.example.test')data=Buffer.concat([Buffer.alloc(4),text('S'),text('RELAY:turn.udp'),text(''),name('_turn._udp.hosted.example.test')]);
      header.writeUInt16BE(packet.readUInt16BE(0),0);header.writeUInt16BE(0x8180,2);header.writeUInt16BE(1,4);
      header.writeUInt16BE(data?1:0,6);
      const record=Buffer.alloc(12);record.writeUInt16BE(0xc00c,0);record.writeUInt16BE(type,2);record.writeUInt16BE(1,4);record.writeUInt16BE(data?.length||0,10);
      const response=Buffer.concat([header,packet.subarray(12),...(data?[record,data]:[])]);
      this.pending++;
      this.socket.send(response,from.port,from.address,error=>{this.pending--;if(error)this.error='controlled DNS reply failed';else this.answers++;});
    });
    await new Promise(resolve=>this.socket.bind(0,'127.0.0.1',resolve));
    this.port=this.socket.address().port;
    return '127.0.0.1:'+this.port;
  }
  snapshot(){return {queries:this.queries,answers:this.answers,held:this.held,types:{...this.types}};}
  async close() {
    await new Promise(resolve=>this.socket.close(resolve));
    const probe=createSocket('udp4');
    try {await new Promise((resolve,reject)=>{probe.once('error',reject);probe.bind(this.port,'127.0.0.1',resolve);});}
    finally {probe.close();}
  }
}

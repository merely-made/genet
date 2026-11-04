// Controlled DNS for ICE-server qualification. Never changes OS DNS or hosts.
import { createSocket } from 'node:dgram';

export class DnsFixture {
  constructor() { this.queries=0;this.answers=0;this.held=0;this.pause=false;this.pending=0;this.error=null; }
  async start(address) {
    const octets=address.split('.').map(Number);
    if(octets.length!==4||octets.some(n=>!Number.isInteger(n)||n<0||n>255))throw Error('DNS fixture requires IPv4');
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
      if(end+4!==packet.length||labels.join('.').toLowerCase()!=='rtc.example.test'||packet.readUInt16BE(end+2)!==1)return;
      this.queries++;
      if(this.pause){this.held++;return;}
      if(this.pending>=64){this.error='controlled DNS reply budget exceeded';return;}
      const type=packet.readUInt16BE(end),header=Buffer.alloc(12);
      header.writeUInt16BE(packet.readUInt16BE(0),0);header.writeUInt16BE(0x8180,2);header.writeUInt16BE(1,4);
      header.writeUInt16BE(type===1?1:0,6);
      const record=Buffer.from([0xc0,0x0c,0,1,0,1,0,0,0,0,0,4,...octets]);
      const response=Buffer.concat([header,packet.subarray(12),...(type===1?[record]:[])]);
      this.pending++;
      this.socket.send(response,from.port,from.address,error=>{this.pending--;if(error)this.error='controlled DNS reply failed';else this.answers++;});
    });
    await new Promise(resolve=>this.socket.bind(0,'127.0.0.1',resolve));
    this.port=this.socket.address().port;
    return '127.0.0.1:'+this.port;
  }
  snapshot(){return {queries:this.queries,answers:this.answers,held:this.held};}
  async close() {
    await new Promise(resolve=>this.socket.close(resolve));
    const probe=createSocket('udp4');
    try {await new Promise((resolve,reject)=>{probe.once('error',reject);probe.bind(this.port,'127.0.0.1',resolve);});}
    finally {probe.close();}
  }
}

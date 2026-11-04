// SPDX-License-Identifier: MPL-2.0
// Native Genet vs installed, headed browsers. No third-party automation library.
// Firefox BiDi: https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection
// Protocol: https://www.w3.org/TR/webdriver-bidi/
// Build (debug=0, jobs=1, stable target/build paths):
//   cargo build -p genet-webrtc --example interop_host --locked --offline -j 1
// Start murm/wire/crates/rtc/tests/coturn.sh in WSL, then from Genet:
//   node support/ci/webrtc_interop.mjs --server NUMERIC_WSL_IP:34790
// Optional matrix filters: --browser chrome|firefox --engine boa|nova
// --route direct|relay --offerer native|browser --cleanup close|navigate.
// --restart both exercises both APIs at every negotiation step; adds configured STUN.
// --bind selects the Windows interface explicitly. Default is the default-route
// IPv4 interface; the UDP fixture front end accepts only local-machine clients.
// Firefox BiDi grants local-network access only to this fixture origin and
// resets it on shutdown. No camera/microphone grant or WebRTC pref override.
// Profiles stay in the marker-owned stable target cache: receiving-environment
// automatic approval review rejected recursive deletion. Live owners forbid reuse.
import { spawn, execFile, execFileSync } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { createServer as tcpServer } from 'node:net';
import { createSocket } from 'node:dgram';
import { readFile, writeFile, mkdir, access, appendFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createInterface } from 'node:readline';
import { runRestartChecks } from './webrtc_restart.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const execFileAsync = promisify(execFile);
const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) args.set(process.argv[i], process.argv[i + 1]);
const restart=args.get('--restart');
if(restart && restart!=='both')throw Error('--restart requires both');
const receiptMode=args.get('--receipt-mode')||'append';
if(!['append','stdout'].includes(receiptMode))throw Error('invalid receipt mode');
const routes=restart?['direct','stun','relay']:['direct','relay'];
const dimensions=[['--browser',['chrome','firefox']],['--engine',['boa','nova']],['--route',routes],['--offerer',['native','browser']],['--cleanup',['close','navigate']]];
for(const [flag,values] of dimensions)if(args.has(flag) && !values.includes(args.get(flag)))throw Error('invalid matrix selector '+flag);
const expectedCases=dimensions.reduce((n,[flag,values])=>n*(args.has(flag)?1:values.length),1);
const target = path.resolve(process.env.CARGO_TARGET_DIR || 'C:/t/cargo-targets/genet');
const scratch = path.join(target, 'rtc-interop');
const nativeExe = args.get('--native') || path.join(target, 'debug/examples/interop_host.exe');
const server = args.get('--server');
if (!server || !/^\d+\.\d+\.\d+\.\d+:\d+$/.test(server)) throw Error('--server requires controlled numeric IPv4:port');
if (process.platform !== 'win32') throw Error('This receiving gate currently qualifies Windows browser/socket ownership');
const [serverIP, serverPort] = server.split(':');
const source = await readFile(path.join(repo, 'support/ci/webrtc_interop_peer.js'), 'utf8');
const sourceFiles=['components/genet-webrtc/examples/interop_host.rs','support/ci/webrtc_interop.mjs','support/ci/webrtc_interop_peer.js','support/ci/webrtc_restart.mjs','components/genet-webrtc/Cargo.toml','components/genet-webrtc/src/lib.rs','components/genet-webrtc/src/signaling.rs','components/genet-webrtc/src/channels.rs','components/genet-webrtc/src/stats.rs','components/script-runtime-api/webrtc/bootstrap.js'];
const receipt = path.join(repo, 'design_docs/archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const hash = data => createHash('sha256').update(data).digest('hex');
const assert = (value, message) => { if (!value) throw Error(message); };
const choose = (key, all) => args.has(key) ? [args.get(key)] : all;
const versions = [], cases = [];
let phase = 'setup', current = null, failure = null;

async function timeout(promise, milliseconds, message) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(Error(message)), milliseconds); })]); }
  finally { clearTimeout(timer); }
}
async function freePort() {
  const socket = tcpServer(); await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve));
  const port = socket.address().port; await new Promise(resolve => socket.close(resolve)); return port;
}
async function bindIP() {
  const socket = createSocket('udp4');
  // A route lookup only: no datagram is sent to this documentation address.
  // The direct WSL endpoint timed out in this receiving environment. Use the
  // Windows-facing default-route interface for both peers and the UDP front end.
  try { await new Promise((resolve,reject) => { socket.once('error',reject); socket.connect(9,'192.0.2.1',resolve); }); return args.get('--bind') || socket.address().address; }
  finally { socket.close(); }
}
class UdpFixture {
  constructor() {this.clients=new Map();this.pending=0;this.bytes=0;this.error=null;}
  send(socket,data,port,address) {
    if(this.pending>=256 || this.bytes+data.length>1048576){this.error='UDP fixture forwarding budget exceeded';return;}
    this.pending++;this.bytes+=data.length;
    socket.send(data,port,address,error=>{this.pending--;this.bytes-=data.length;if(error)this.error='UDP fixture send '+error.code;});
  }
  async start(bind) {
    this.front=createSocket('udp4');this.front.on('error',()=>{this.error='UDP fixture front socket failed';});
    this.front.on('message',(data,from)=>{
      if(from.address!==bind && from.address!=='127.0.0.1')return;
      const key=from.address+':'+from.port;let socket=this.clients.get(key);
      if(!socket) {
        if(this.clients.size>=16){this.error='UDP fixture client budget exceeded';return;}
        socket=createSocket('udp4');this.clients.set(key,socket);
        socket.on('error',()=>{this.error='UDP fixture backend socket failed';});
        socket.on('message',(reply,peer)=>{
          if(peer.address===serverIP && peer.port===Number(serverPort))this.send(this.front,reply,from.port,from.address);
        });
      }
      this.send(socket,data,Number(serverPort),serverIP);
    });
    await new Promise(resolve=>this.front.bind(0,bind,resolve));
    return bind+':'+this.front.address().port;
  }
  async clear() {
    for(let i=0;i<100 && this.pending;i++)await delay(10);
    assert(!this.pending,'UDP fixture send queue did not drain');
    for(const socket of this.clients.values())socket.close();this.clients.clear();
  }
  async close(){await this.clear();await new Promise(resolve=>this.front.close(resolve));}
}
function powershell(script) {
  return execFileSync('C:/Program Files/PowerShell/7/pwsh.exe', ['-NoProfile','-NonInteractive','-Command',script],
    {encoding:'utf8',windowsHide:true,timeout:15000,maxBuffer:65536}).trim();
}
async function ownedUdp(pid) {
  // Keep forwarding STUN/TURN datagrams while Windows measures socket ownership.
  const {stdout} = await execFileAsync('C:/Program Files/PowerShell/7/pwsh.exe',
    ['-NoProfile','-NonInteractive','-Command',`@(Get-NetUDPEndpoint -OwningProcess ${pid} -ErrorAction SilentlyContinue | ForEach-Object { @{address=$_.LocalAddress;port=$_.LocalPort} }) | ConvertTo-Json -Compress`],
    {encoding:'utf8',windowsHide:true,timeout:15000,maxBuffer:65536});
  const text=stdout.trim();
  if (!text) return []; const result = JSON.parse(text); return Array.isArray(result) ? result : [result];
}
async function released(routes) {
  for (const route of routes) {
    const socket = createSocket('udp4');
    try { await new Promise((resolve,reject) => { socket.once('error',reject); socket.bind(route.port,route.address,resolve); }); }
    finally { socket.close(); }
  }
}
function visibleWindows(pid) {
  const code = `import ctypes\nfrom ctypes import wintypes as w\nu=ctypes.windll.user32\nu.IsWindowVisible.argtypes=[w.HWND]\nu.GetWindowThreadProcessId.argtypes=[w.HWND,ctypes.POINTER(w.DWORD)]\nhits=[]\ndef visit(hwnd,lparam):\n p=w.DWORD()\n u.GetWindowThreadProcessId(hwnd,ctypes.byref(p))\n if p.value==${pid} and u.IsWindowVisible(hwnd):hits.append(1)\n return True\ncallback=ctypes.WINFUNCTYPE(w.BOOL,w.HWND,w.LPARAM)(visit)\nu.EnumWindows(callback,0)\nprint(len(hits))`;
  return Number(execFileSync('python',['-c',code],{encoding:'utf8',windowsHide:true,timeout:10000}));
}
class SocketRpc {
  constructor(socket) {
    this.socket=socket; this.next=0; this.pending=new Map();
    socket.addEventListener('message',event=>{
      const message=JSON.parse(event.data), item=this.pending.get(message.id);
      if (!item) return; this.pending.delete(message.id);
      message.error ? item.reject(Error('browser RPC '+message.error)) : item.resolve(message.result);
    });
    socket.addEventListener('close',()=>{ for(const item of this.pending.values())item.reject(Error('browser RPC closed')); this.pending.clear(); });
  }
  call(method,params={}) {
    const id=++this.next;
    return timeout(new Promise((resolve,reject)=>{ this.pending.set(id,{resolve,reject});this.socket.send(JSON.stringify({id,method,params})); }),15000,'browser RPC deadline');
  }
}
async function connect(url) {
  for(let attempt=0;attempt<100;attempt++) {
    const socket=new WebSocket(url);
    try { await timeout(new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});}),1000,'debug endpoint deadline'); return new SocketRpc(socket); }
    catch { socket.close(); await delay(100); }
  }
  throw Error('browser automation endpoint unavailable');
}
class Browser {
  async start(name,url) {
    this.name=name;this.profile=path.resolve(scratch,name+'-profile');
    assert(path.dirname(this.profile)===path.resolve(scratch),'profile outside owned scratch');
    try {
      await access(this.profile);
      const previous=JSON.parse(await readFile(path.join(this.profile,'genet-rtc-owner.json'),'utf8'));
      assert(previous.kind==='genet-rtc-interop','existing directory is not a marked test profile');
      let alive=true;try{process.kill(previous.owner,0);}catch(e){if(e.code==='ESRCH')alive=false;else throw e;}
      assert(!alive,'previous profile owner remains alive');
      const matching=powershell(`@(Get-CimInstance Win32_Process | Where-Object { $_.Name -in @('chrome.exe','firefox.exe') -and $_.CommandLine -like '*${this.profile.replaceAll("'","''")}*' }).Count`);
      assert(Number(matching)===0,'browser still owns the test profile');
    } catch(e) { if(e.code!=='ENOENT')throw e; }
    await mkdir(this.profile,{recursive:true});
    await writeFile(path.join(this.profile,'genet-rtc-owner.json'),JSON.stringify({kind:'genet-rtc-interop',owner:process.pid}));
    const port=await freePort();
    const executable=name==='chrome' ? args.get('--chrome') || 'C:/Program Files/Google/Chrome/Application/chrome.exe' :
      args.get('--firefox') || 'C:/Program Files/Mozilla Firefox/firefox.exe';
    let flags;
    if(name==='chrome') flags=[`--user-data-dir=${this.profile}`,`--remote-debugging-port=${port}`,'--no-first-run','--no-default-browser-check','--disable-background-networking','about:blank'];
    else {
      const prefs={'browser.shell.checkDefaultBrowser':false,'browser.startup.homepage_override.mstone':'ignore','browser.startup.homepage':'about:blank','datareporting.policy.dataSubmissionPolicyBypassNotification':true};
      await writeFile(path.join(this.profile,'user.js'),Object.entries(prefs).map(([k,v])=>`user_pref(${JSON.stringify(k)},${JSON.stringify(v)});`).join('\n'));
      flags=['-no-remote','-profile',this.profile,'--remote-debugging-port',String(port),'about:blank'];
    }
    this.process=spawn(executable,flags,{stdio:['ignore','ignore','pipe'],windowsHide:true});
    this.stderr='';this.networkTrace=[];this.process.stderr.on('data',chunk=>{
      this.stderr=(this.stderr+chunk.toString()).slice(-8192);
      if(process.env.RTC_INTEROP_TRACE==='1')for(const line of chunk.toString().split(/\r?\n/))
        if(/mtransport|nicer/.test(line) && /error|fail|socket|STUN|TURN/i.test(line) && !/pwd|ufrag|password|username|credential|sdp/i.test(line)) {
          this.networkTrace.push(line.replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g,'<ip>').slice(0,240));
          if(this.networkTrace.length>24)this.networkTrace.shift();
        }
    });
    this.exit=new Promise(resolve=>this.process.once('exit',resolve));
    if(name==='chrome') {
      let endpoint;
      for(let i=0;i<100;i++) {
        try { const response=await fetch(`http://127.0.0.1:${port}/json/new?${encodeURIComponent(url)}`,{method:'PUT'}); if(response.ok){endpoint=await response.json();break;} } catch {}
        await delay(100);
      }
      assert(endpoint,'Chrome debug tab unavailable');
      this.rpc=await connect(endpoint.webSocketDebuggerUrl);
      await this.rpc.call('Page.enable');
      const version=await this.rpc.call('Browser.getVersion'); this.version=version.product;
    } else {
      this.rpc=await connect(`ws://127.0.0.1:${port}/session`);
      const session=await this.rpc.call('session.new',{capabilities:{}});
      this.version=session.capabilities.browserVersion;
      this.context=(await this.rpc.call('browsingContext.create',{type:'tab'})).context;
      await this.rpc.call('browsingContext.navigate',{context:this.context,url,wait:'complete'});
    }
    this.networkPermissions=[];this.permissionOrigin=new URL(url).origin;
    if(name==='firefox') {
      // LNA is an ordinary per-origin browser permission, separate from media.
      // Grant only this disposable localhost fixture's origin through BiDi.
      for(const permission of ['local-network','loopback-network','local-network-access']) {
        const query=`(async()=>{try{return (await navigator.permissions.query({name:${JSON.stringify(permission)}})).state;}catch{return 'unsupported';}})()`;
        const before=await this.evaluate(query,true);
        if(before==='unsupported')continue;
        let error=null;
        try { await this.rpc.call('permissions.setPermission',{descriptor:{name:permission},state:'granted',origin:new URL(url).origin}); }
        catch(e){error=e.message;}
        this.networkPermissions.push({permission,before,after:await this.evaluate(query,true),error});
      }
    }
    this.windowPid=this.process.pid;
    if(!visibleWindows(this.windowPid)) {
      // Firefox's Windows launcher can hand off to a child browser process.
      const rows=JSON.parse(powershell(`@(Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'firefox.exe' -and ($_.ParentProcessId -eq ${this.process.pid} -or $_.CommandLine -like '*${this.profile.replaceAll("'","''")}*') } | ForEach-Object { $_.ProcessId }) | ConvertTo-Json -Compress`) || '[]');
      for(const pid of Array.isArray(rows)?rows:[rows])if(visibleWindows(pid)){this.windowPid=pid;break;}
    }
    assert(visibleWindows(this.windowPid)>0,'browser has no OS-visible owned window');
    versions.push({browser:name,version:this.version,headed:true,visibleWindows:visibleWindows(this.windowPid),webrtcPreferenceOverrides:false,networkPermissions:this.networkPermissions});
  }
  async evaluate(expression,awaitPromise=false) {
    if(this.name==='chrome') {
      const result=await this.rpc.call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise});
      assert(!result.exceptionDetails,'Chrome fixture evaluation failed');return result.result.value;
    }
    const result=await this.rpc.call('script.evaluate',{expression,target:{context:this.context},awaitPromise});
    assert(result.type==='success','Firefox fixture evaluation failed');return result.result.value;
  }
  async value(expression) { return JSON.parse(await this.evaluate(`JSON.stringify(${expression})`)); }
  async reset(url) {
    if(this.name==='chrome') { await this.rpc.call('Page.navigate',{url}); }
    else await this.rpc.call('browsingContext.navigate',{context:this.context,url,wait:'complete'});
    for(let i=0;i<100;i++) { if(await this.value(`location.href===${JSON.stringify(url)} && document.readyState==='complete' && document.title==='RTC interoperability fixture'`))return;await delay(25); }
    throw Error('browser fixture navigation deadline');
  }
  async close() {
    if(this.rpc) {
      if(process.env.RTC_INTEROP_TRACE==='1' && this.networkTrace?.length)console.error(JSON.stringify({firefoxNetworkTrace:this.networkTrace}));
      if(this.name==='firefox')for(const permission of this.networkPermissions||[]) {
        if(permission.error)continue;
        try {await this.rpc.call('permissions.setPermission',{descriptor:{name:permission.permission},state:'prompt',origin:this.permissionOrigin});permission.reset='prompt';}catch{permission.reset='failed';}
      }
      try { await this.rpc.call(this.name==='chrome'?'Browser.close':'browser.close'); } catch {}
      this.rpc.socket.close();
    }
    if(this.process && this.process.exitCode===null) {
      try { await timeout(this.exit,5000,'browser shutdown deadline'); }
      catch { try {execFileSync('taskkill',['/PID',String(this.process.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});}catch{} await timeout(this.exit,5000,'browser still alive after shutdown'); }
    }
    if(this.windowPid && this.windowPid!==this.process.pid) {
      for(let attempt=0;attempt<100;attempt++) {
        try {process.kill(this.windowPid,0);}catch(e){if(e.code==='ESRCH')break;throw e;}
        await delay(50);
        assert(attempt!==99,'owned Firefox window process did not exit');
      }
    }
    if(this.profile) {
      const marker=JSON.parse(await readFile(path.join(this.profile,'genet-rtc-owner.json'),'utf8'));
      assert(marker.kind==='genet-rtc-interop' && marker.owner===process.pid,'profile owner mismatch');
      assert(path.dirname(path.resolve(this.profile))===path.resolve(scratch),'profile cleanup outside owned scratch');
      // Retain this bounded, marked cache: automatic approval review rejected
      // recursive profile cleanup in the receiving environment. Reuse requires
      // both the previous controller and its browser to have exited.
    }
  }
}
class Native {
  start(engine,bind) {
    this.process=spawn(nativeExe,[engine,bind],{stdio:['pipe','pipe','pipe'],windowsHide:true});
    this.next=0;this.pending=new Map();this.stderr='';
    this.ready=new Promise(resolve=>{this.markReady=resolve;});
    this.process.stderr.on('data',chunk=>{this.stderr=(this.stderr+chunk.toString()).slice(-4096);});
    this.exit=new Promise(resolve=>this.process.once('exit',code=>{for(const item of this.pending.values())item.reject(Error('native fixture exited '+code));this.pending.clear();resolve(code);}));
    createInterface({input:this.process.stdout}).on('line',line=>{
      assert(line.length<262144,'native control response exceeded budget');
      const response=JSON.parse(line);if(response.ready){this.markReady();return;}const item=this.pending.get(response.id);if(!item)return;this.pending.delete(response.id);
      response.ok?item.resolve(response.value):item.reject(Error(response.error));
    });
  }
  request(op,extra={}) {
    const id=++this.next;return timeout(new Promise((resolve,reject)=>{this.pending.set(id,{resolve,reject});this.process.stdin.write(JSON.stringify({id,op,...extra})+'\n');}),15000,'native control deadline');
  }
  evaluate(source) { return this.request('eval',{source}); }
  async value(expression) { return JSON.parse(await this.evaluate(`JSON.stringify(${expression})`)); }
  async close() {
    if(this.process.exitCode===null) {this.process.stdin.end(JSON.stringify({op:'quit'})+'\n');try{await timeout(this.exit,2000,'native shutdown deadline');}catch{this.process.kill();await this.exit;}}
  }
}
function verifySnapshot(snapshot,other,connected=true) {
  assert(!snapshot.errors.length,'peer error '+snapshot.errors.join(','));
  assert((!connected||snapshot.connected) && snapshot.sent.length===3 && snapshot.received.length===3,'incomplete application exchange');
  assert(snapshot.received.includes(other+':text:λ🙂:reliable:'+snapshot.round) && snapshot.received.includes(other+':text:λ🙂:unreliable:'+snapshot.round) && snapshot.received.includes('binary:48000:'+snapshot.round),'message content mismatch');
  assert(snapshot.modes.some(c=>c.label==='reliable' && c.ordered && c.maxRetransmits===null),'reliable mode mismatch');
  assert(snapshot.modes.some(c=>c.label==='unreliable' && !c.ordered && c.maxRetransmits===0),'partial reliability mismatch');
}
function verifyStats(a,b,route) {
  assert(a.candidateReferencesValid===true,'native stats retained a dangling candidate reference');
  assert(a.remoteAddressesRedacted===true && a.peerReflexiveInRemoteList===false,'native ICE observation exposed an unsignaled address');
  for(const stats of [a,b]) {
    assert(stats.pair && stats.pair.state==='succeeded','selected candidate pair unavailable');
    assert(stats.pair.localProtocol==='udp' && stats.pair.remoteProtocol==='udp','unexpected selected transport');
    if(route==='relay')assert(stats.pair.local==='relay' && stats.pair.remote==='relay','forced relay used a direct candidate');
    else assert(stats.pair.local!=='relay' && stats.pair.remote!=='relay','direct case used relay');
  }
  assert(a.remoteFingerprint===b.localFingerprint && b.remoteFingerprint===a.localFingerprint,'SDP fingerprint mismatch');
  for(const [stats,other] of [[a,b],[b,a]]) {
    const hashes=stats.remoteCertificates.map(bytes=>hash(Buffer.from(bytes)));
    assert(hashes.includes(other.localFingerprint) || stats.statsRemoteFingerprint===other.localFingerprint,'remote certificate was not independently observed');
    stats.remoteDER_SHA256=hashes;delete stats.remoteCertificates;
  }
}

const sourceAtStart={};for(const file of sourceFiles)sourceAtStart[file]=hash(await readFile(path.join(repo,file)));
const binaryAtStart=hash(await readFile(nativeExe));
const lockAtStart=hash(await readFile(path.join(repo,'Cargo.lock')));
const http=createServer((request,response)=>{response.writeHead(200,{'Content-Type':'text/html','Cache-Control':'no-store'});response.end('<!doctype html><title>RTC interoperability fixture</title><h1>RTC interoperability fixture</h1><p>Automated data-channel peer. No media capture.</p>');});
await new Promise(resolve=>http.listen(0,'127.0.0.1',resolve));
const url=`http://127.0.0.1:${http.address().port}/`;
const bind=await bindIP();
const udpFixture=new UdpFixture();
const frontend=await udpFixture.start(bind);
try {
  for(const browserName of choose('--browser',['chrome','firefox'])) {
    const browser=new Browser();
    try {
      phase='browser startup';await browser.start(browserName,url);
      for(const engine of choose('--engine',['boa','nova']))for(const route of choose('--route',routes))
      for(const offerer of choose('--offerer',['native','browser']))for(const cleanup of choose('--cleanup',['close','navigate'])) {
        current={browser:browserName,engine,route,offerer,cleanup}; const native=new Native();const start=Date.now();let a,b;
        try {
          phase='peer startup';await browser.reset(url+'?case='+cases.length);native.start(engine,bind);await timeout(native.ready,45000,'native initialization deadline');
          const config={iceTransportPolicy:route==='relay'?'relay':'all',iceServers:route==='relay'?
            [{urls:[`turn:${frontend}?transport=udp`],username:'rtc-test',credential:'rtc-test-password'}]:route==='stun'?[{urls:[`stun:${frontend}`]}]:[]};
          await native.evaluate(source);await browser.evaluate(source);
          await native.evaluate(`globalThis.__interop=makeInteropPeer(${JSON.stringify(config)},'native',${offerer==='native'});__interop.start()`);
          await browser.evaluate(`globalThis.__interop=makeInteropPeer(${JSON.stringify(config)},'browser',${offerer==='browser'});__interop.start()`);
          phase='negotiation and transfer';const exchangeStart=Date.now();
          for(let i=0;i<600;i++) {
            for(const signal of await native.value('__interop.takeSignals()'))await browser.evaluate(`__interop.receive(${JSON.stringify(signal)})`);
            for(const signal of await browser.value('__interop.takeSignals()'))await native.evaluate(`__interop.receive(${JSON.stringify(signal)})`);
            a=await native.value('__interop.snapshot()');b=await browser.value('__interop.snapshot()');
            assert(!udpFixture.error,udpFixture.error);
            assert(!a.errors.length && !b.errors.length,'peer error '+[...a.errors,...b.errors].join(','));
            if(a.connected && b.connected && a.received.length===3 && b.received.length===3)break;
            assert(Date.now()-exchangeStart<25000,'application exchange deadline');await delay(15);
          }
          verifySnapshot(a,'browser');verifySnapshot(b,'native');
          phase='route and certificate verification';await native.evaluate('__interop.inspect()');await browser.evaluate('__interop.inspect()');
          let aStats,bStats;
          for(let i=0;i<100;i++){aStats=await native.value('__interop.stats');bStats=await browser.value('__interop.stats');if(aStats&&bStats)break;await delay(15);}
          assert(aStats&&bStats,'stats deadline');verifyStats(aStats,bStats,route);
          const initialSockets=restart?await ownedUdp(native.process.pid):null;
          let restarts;
          if(restart) {
            phase='ICE restart';restarts=await runRestartChecks({native,browser,udpFixture,offerer,route,verifySnapshot,verifyStats});
            a=await native.value('__interop.snapshot()');b=await browser.value('__interop.snapshot()');
            aStats=await native.value('__interop.stats');bStats=await browser.value('__interop.stats');verifyStats(aStats,bStats,route);
            phase='restart interruption';
            await native.evaluate('__interop.captureRestart();__interop.restart("method")');
            const deadline=Date.now()+15000;let pending;
            while(Date.now()<deadline){pending=await native.value('__interop.snapshot()');if(pending.signaling==='have-local-offer'&&pending.localCredentialsChanged)break;await delay(10);}
            assert(pending.signaling==='have-local-offer'&&pending.localCredentialsChanged,'interrupted restart did not apply an offer');
          }
          phase='native cleanup';const sockets=await ownedUdp(native.process.pid);assert(sockets.length===1,'expected one owned RTC socket');
          if(restart)assert(initialSockets.length===1&&sockets[0].address===initialSockets[0].address&&sockets[0].port===initialSockets[0].port,'restart replaced or leaked the native socket');
          let cleaned;
          if(cleanup==='navigate') {
            cleaned=await native.request('navigate');assert(cleaned.pending===0,'navigation retained a peer');
            assert(cleaned.stale[0]==='closed' && cleaned.stale.slice(1,3).every(x=>x==='closed') && cleaned.stale.slice(-2).every(x=>x==='InvalidStateError'),'navigation retained live RTC handles');
          } else {
            cleaned=await native.value('__interop.close()');assert(cleaned.state==='closed' && cleaned.channels.every(x=>x==='closed'),'close left live wrappers');
            assert((await native.request('metrics')).pending===0,'close retained a peer');
          }
          assert((await ownedUdp(native.process.pid)).length===0,'native cleanup retained UDP ownership');await released(sockets);
          await browser.evaluate('__interop.close()');
          const summary=(snapshot,stats)=>({connected:snapshot.connected,textMessagesVerified:2,binaryBytesVerified:48000,modes:snapshot.modes,refused:snapshot.refused,gatheringErrors:snapshot.gatheringErrors,
            pair:stats.pair,localFingerprint:stats.localFingerprint,remoteFingerprint:stats.remoteFingerprint,remoteDER_SHA256:stats.remoteDER_SHA256,statsRemoteFingerprint:stats.statsRemoteFingerprint,
            remoteAddressesRedacted:stats.remoteAddressesRedacted,peerReflexiveInRemoteList:stats.peerReflexiveInRemoteList});
          cases.push({...current,status:'pass',seconds:Math.round((Date.now()-start)/10)/100,native:summary(a,aStats),browserPeer:summary(b,bStats),restarts,interruptedRestart:restart?true:undefined,socketPreserved:restart?true:undefined,cleanupVerified:true,socketRebound:true});
          console.log(JSON.stringify({...current,status:'pass',seconds:cases.at(-1).seconds}));
        } catch(error) {
          try { await native.evaluate('__interop.inspect()'); await browser.evaluate('__interop.inspect()'); await delay(100); } catch {}
          let observedStats; try {observedStats={native:await native.value('__interop.stats'),browser:await browser.value('__interop.stats')};} catch {}
          for(const stats of Object.values(observedStats||{}))if(stats){stats.remoteDER_SHA256=(stats.remoteCertificates||[]).map(bytes=>hash(Buffer.from(bytes)));delete stats.remoteCertificates;}
          cases.push({...current,status:'fail',phase,error:error.message.slice(0,180),observed:{native:a,browser:b},observedStats,restartDiagnostics:error.restartDiagnostics,trace:native.stderr.split('\n').filter(s=>s.startsWith('RTC poll variant:')).slice(-8)});throw error;
        } finally {
          try {await browser.evaluate('globalThis.__interop?.close()');}catch{}
          await native.close();await delay(100);await udpFixture.clear();
        }
      }
    } finally {await browser.close();}
  }
} catch(error) {failure={phase,case:current,error:error.message.slice(0,180)};console.error(JSON.stringify(failure));process.exitCode=1;}
finally {
  await udpFixture.close();
  await new Promise(resolve=>http.close(resolve));
  const hashes={};for(const file of sourceFiles)hashes[file]=hash(await readFile(path.join(repo,file)));
  const sourceStable=JSON.stringify(hashes)===JSON.stringify(sourceAtStart)&&binaryAtStart===hash(await readFile(nativeExe));
  if(!sourceStable){failure={phase:'source identity',error:'source or fixture binary changed during qualification'};process.exitCode=1;}
  const gate=restart?'E2r':'E2h';
  const record={schema:1,gate,sourceStable,lockSHA256:lockAtStart,utc:new Date().toISOString(),source:execFileSync('git',['rev-parse','HEAD'],{cwd:repo,encoding:'utf8'}).trim(),node:process.version,command:['node','support/ci/webrtc_interop.mjs',...process.argv.slice(2)],expectedCases,completeMatrix:expectedCases===(restart?48:32)&&cases.length===expectedCases&&!failure,nativeBinarySHA256:hash(await readFile(nativeExe)),sourceSHA256:hashes,versions,cases,failure,relayFixture:'bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 16 clients/256 pending datagrams/1 MiB',profileCache:scratch,profileCleanup:'automatic approval review blocked recursive deletion; marked profiles retained for reuse',scope:restart?'native Windows headed data-channel restart on fixed direct/STUN/UDP-TURN routes; DNS/mDNS, server replacement, TCP/TLS, interface rebinding, media and carrier acceptance remain open':'native Windows headed data channels, controlled WSL UDP server'};
  if(receiptMode==='stdout') {
    console.log(`${gate} ${cases.filter(c=>c.status==='pass').length}/${cases.length} passed; receipt emitted`);
    console.log('RTC_RECEIPT '+JSON.stringify(record));
  } else {
  const old=await readFile(receipt,'utf8'),marker='\n\n## '+gate+' automated browser interoperability';
  const laneStart=old.indexOf(restart?'\n\n## E2r ':'\n\n## E2h automated browser interoperability');
  const base=laneStart<0?Buffer.byteLength(old):Buffer.byteLength(old.slice(0,laneStart));
  const entry=marker+' - '+record.utc+'\n\n```json\n'+JSON.stringify(record)+'\n```\n';
  const laneEnd=restart?-1:old.indexOf('\n\n## E2r ',laneStart);
  const historical=laneEnd<0?Buffer.byteLength(old)-base:Buffer.byteLength(old.slice(laneStart,laneEnd));
  assert(Buffer.byteLength(old+entry)<=1000000 && historical+Buffer.byteLength(entry)<=(restart?220000:256000),gate+' receipt byte cap exceeded');
  await appendFile(receipt,entry);console.log(`${gate} ${cases.filter(c=>c.status==='pass').length}/${cases.length} passed; compact receipt appended`);
  }
}

// SPDX-License-Identifier: MPL-2.0
// Native Genet vs installed, headed browsers. No third-party automation library.
// Firefox BiDi: https://developer.mozilla.org/en-US/docs/Web/WebDriver/How_to/Create_BiDi_connection
// Protocol: https://www.w3.org/TR/webdriver-bidi/
// Build (debug=0, jobs=1, stable target/build paths):
//   cargo build -p genet-webrtc --example interop_host --locked --offline -j 1
// Start murm/wire/crates/rtc/tests/coturn.sh in WSL, then from Genet:
//   node support/ci/webrtc_interop.mjs --server NUMERIC_WSL_IP:34790
// Add --dns controlled for explicit-port native hostname URLs, controlled A
// replies, and interrupted-DNS cleanup. Browser peers use the numeric endpoint.
// --dns service instead uses portless URLs and controlled SRV/S-NAPTR replies.
// --local-mdns enabled tests native advertisement without rewriting native names.
// Direct initial browser candidates are delayed to prove browser name resolution.
// --mdns-alias-browser firefox explicitly tests its local sockets with fixture
// names, leaving that browser's own mDNS advertisement gate unqualified.
// Optional matrix filters: --browser chrome|firefox --engine boa|nova
// --route direct|relay --offerer native|browser --cleanup close|navigate.
// --restart both exercises both APIs at every negotiation step; adds configured STUN.
// Restart append mode checkpoints each case. --resume latest continues a run
// only with identical source, native binary, lock, server/interface and browsers.
// --answer-delay-ms 0..10000 holds restart answers for the bounded late-check
// regression. The delay is part of checkpoint identity.
// --bind selects the Windows interface explicitly. Default is the default-route
// IPv4 interface; the UDP fixture front end accepts only local-machine clients.
// Firefox BiDi grants local-network access only to this fixture origin and
// resets it on shutdown. No camera/microphone grant or WebRTC pref override.
// Profiles stay in the marker-owned stable target cache: receiving-environment
// earlier cleanup was rejected as blocked by policy; reviewer not established. Live owners forbid reuse.
import { spawn, execFile, execFileSync } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { createServer as tcpServer, isIPv4 } from 'node:net';
import { createSocket } from 'node:dgram';
import { readFile, writeFile, mkdir, access, appendFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createInterface } from 'node:readline';
import { runRestartChecks } from './webrtc_restart.mjs';
import { RestartJournal, assertWritablePacket } from './webrtc_receipts.mjs';
import { DnsFixture } from './webrtc_dns_fixture.mjs';
import { settledStats } from './webrtc_stats.mjs';
import { inspectAdvertisement } from './webrtc_advertisement.mjs';
import { MdnsFixture } from './webrtc_mdns_fixture.mjs';
import { FirefoxIpcTrace } from './webrtc_firefox_ipc.mjs';
import { FirefoxTrace } from './webrtc_firefox_trace.mjs';
import { probeRegistration, observedBoundQueryKinds } from './webrtc_registration_probe.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const execFileAsync = promisify(execFile);
const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) args.set(process.argv[i], process.argv[i + 1]);
const restart=args.get('--restart');
const dnsMode=args.get('--dns')||null;
const dnsEnabled=dnsMode!==null;
const continueOnFailure=args.get('--continue-on-failure')==='yes';
if(args.has('--continue-on-failure')&&!continueOnFailure)throw Error('--continue-on-failure requires yes');
const localMdns=args.get('--local-mdns')==='enabled';
if(args.has('--local-mdns')&&!localMdns)throw Error('--local-mdns requires enabled');
if(localMdns&&restart!=='both')throw Error('--local-mdns requires --restart both');
const mdnsEnabled=args.get('--mdns')==='controlled';
const mdnsAliasBrowser=args.get('--mdns-alias-browser')||null;
if(mdnsAliasBrowser&&((!mdnsEnabled&&!localMdns)||mdnsAliasBrowser!=='firefox'))throw Error('--mdns-alias-browser requires mDNS and a named installed browser');
if(args.has('--mdns')&&!mdnsEnabled)throw Error('--mdns requires controlled');
if(localMdns&&(mdnsEnabled||dnsEnabled))throw Error('advertisement gate uses numeric server URLs and untouched browser names');
if(mdnsEnabled&&restart!=='both')throw Error('--mdns requires --restart both');
if(dnsEnabled&&!['controlled','service'].includes(dnsMode))throw Error('--dns requires controlled or service');
const restartDiagnostic=args.get('--restart-diagnostic')||null;
if(restartDiagnostic&&(!localMdns||args.get('--route')!=='direct'||!['observe','defer-end','trickle','trickle-option'].includes(restartDiagnostic)))throw Error('restart diagnostic requires local mDNS, direct route and a supported control');
const firefoxLog=args.get('--firefox-log')||null;
if(firefoxLog&&(firefoxLog!=='counters'||!restartDiagnostic||args.get('--browser')!=='firefox'))throw Error('Firefox log counters require a Firefox restart diagnostic');
const firefoxIpc=args.get('--firefox-ipc')==='yes';
if(args.has('--firefox-ipc')&&(!firefoxIpc||!firefoxLog))throw Error('Firefox IPC tracing requires log counters and yes');
const mdnsObserveInterface=args.get('--mdns-observe-interface')||null;
if(mdnsObserveInterface&&(!restartDiagnostic||!isIPv4(mdnsObserveInterface)))throw Error('additional mDNS observation requires an IPv4 interface and restart diagnostic');
const registrationProbes=args.get('--registration-probes')==='yes';
if(args.has('--registration-probes')&&(!registrationProbes||!restartDiagnostic||!mdnsAliasBrowser))throw Error('registration probes require a restart diagnostic and its mDNS fixture');
const gate=localMdns?'E2ah':mdnsEnabled?'E2m':dnsMode==='service'?'E2s':dnsEnabled?'E2d':restart?'E2r':'E2h';
if(restart && restart!=='both')throw Error('--restart requires both');
// Diagnostic signaling delay, bounded below the retained-route deadline.
const answerDelayMs=Number(args.get('--answer-delay-ms')||0);
if(!Number.isSafeInteger(answerDelayMs)||answerDelayMs<0||answerDelayMs>10000||(!restart&&answerDelayMs))throw Error('invalid restart answer delay');
const receiptMode=args.get('--receipt-mode')||'append';
if(!['append','stdout'].includes(receiptMode))throw Error('invalid receipt mode');
if(args.has('--resume')&&(!restart||receiptMode!=='append'))throw Error('--resume requires --restart both and append mode');
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
const sourceFiles=['support/ci/webrtc_firefox_ipc.mjs','support/ci/webrtc_firefox_ipc.test.mjs','support/ci/webrtc_registration_probe.mjs','support/ci/webrtc_registration_probe.test.mjs','support/ci/webrtc_host_evidence.test.mjs','support/ci/webrtc_firefox_trace.mjs','support/ci/webrtc_firefox_trace.test.mjs','support/ci/webrtc_restart_signals.mjs','support/ci/webrtc_restart_signals.test.mjs','components/genet-webrtc/src/local_descriptions.rs','support/ci/webrtc_mdns_observer.test.mjs','support/ci/webrtc_mdns_observer.mjs','support/ci/webrtc_advertisement.mjs','support/ci/webrtc_advertisement.test.mjs','components/genet-webrtc/src/local_mdns.rs','support/ci/webrtc_permissions.ps1','support/ci/webrtc_mdns_fixture.test.mjs','components/genet-webrtc/src/mdns.rs','support/ci/webrtc_mdns_fixture.mjs','support/ci/webrtc_stats.mjs','support/ci/webrtc_stats.test.mjs','support/ci/webrtc_dns_fixture.test.mjs','components/genet-webrtc/tests/browser/service.rs','components/genet-webrtc/tests/browser/dns_fixture.rs','components/genet-webrtc/examples/interop_host.rs','components/genet-webrtc/examples/interop_host/trace.rs','support/ci/webrtc_interop.mjs','support/ci/webrtc_interop_peer.js','support/ci/webrtc_restart.mjs','support/ci/webrtc_receipts.mjs','components/genet-webrtc/Cargo.toml','components/genet-webrtc/src/lib.rs','components/genet-webrtc/src/discovery.rs','support/ci/webrtc_dns_fixture.mjs','components/genet-webrtc/src/signaling.rs','components/genet-webrtc/src/channels.rs','components/genet-webrtc/src/stats.rs','components/script-runtime-api/webrtc/bootstrap.js'];
const receipt=path.resolve(args.get('--receipt')||path.join(repo,'../../testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_mdns_plan_receipts.md'));
if(['2026-11-01','2026-11-03'].some(date=>receipt.endsWith(date+'_webrtc_conformance_plan_receipts.md')))throw Error('baseline receipt is frozen');
if(receiptMode==='append')assertWritablePacket(await readFile(receipt,'utf8'));
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
    // Port 5353 is deliberately shared with browser/system responders. Native
    // ownership is independently checked as zero before this shared rebind.
    const socket = createSocket({type:'udp4',reuseAddr:route.port===5353});
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
    const browserEnv={...process.env};
    if(firefoxIpc)browserEnv.MOZ_REMOTE_ALLOW_SYSTEM_ACCESS='1';
    if(firefoxLog){delete browserEnv.MOZ_LOG_FILE;delete browserEnv.NSPR_LOG_FILE;delete browserEnv.RUST_LOG;browserEnv.MOZ_LOG='timestamp,signaling:5,mtransport:5,mdns_service:5,nsHostResolver:5,nsDNSService:5';this.firefoxTrace=new FirefoxTrace();}
    this.process=spawn(executable,flags,{stdio:['ignore',firefoxLog?'pipe':'ignore','pipe'],windowsHide:true,env:browserEnv});
    this.stderr='';this.networkTrace=[];this.process.stderr.on('data',chunk=>{
      this.firefoxTrace?.push(chunk);
      this.stderr=(this.stderr+chunk.toString()).slice(-8192);
      if(!firefoxLog&&process.env.RTC_INTEROP_TRACE==='1')for(const line of chunk.toString().split(/\r?\n/))
        if(/mtransport|nicer/.test(line) && /error|fail|socket|STUN|TURN/i.test(line) && !/pwd|ufrag|password|username|credential|sdp/i.test(line)) {
          this.networkTrace.push(line.replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g,'<ip>').slice(0,240));
          if(this.networkTrace.length>24)this.networkTrace.shift();
        }
    });
    this.process.stdout?.on('data',chunk=>this.firefoxTrace?.pushStdout(chunk));
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
    if(firefoxIpc){this.ipcTrace=new FirefoxIpcTrace(this.rpc);await this.ipcTrace.start();}
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
      if(this.ipcTrace)await this.ipcTrace.close();
      if(!firefoxLog&&process.env.RTC_INTEROP_TRACE==='1' && this.networkTrace?.length)console.error(JSON.stringify({firefoxNetworkTrace:this.networkTrace}));
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
      // Retain this bounded, marked cache: a cleanup command was rejected as blocked by policy; reviewer not established.
      // Preserve the marked profiles in the receiving environment. Reuse requires
      // both the previous controller and its browser to have exited.
    }
  }
}
class Native {
  start(engine,bind,dns) {
    this.process=spawn(nativeExe,[engine,bind,...(dns?[dns]:[])],{stdio:['pipe','pipe','pipe'],windowsHide:true,env:{...process.env,GENET_RTC_LOCAL_MDNS:localMdns?'1':'0'}});
    this.next=0;this.pending=new Map();this.stderr='';
    this.process.stdin.on('error',()=>{for(const item of this.pending.values())item.reject(Error('native control pipe closed'));this.pending.clear();});
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
    if(this.process.exitCode!==null||this.process.stdin.destroyed)return Promise.reject(Error('native fixture is closed'));
    const kind=op==='eval'?(extra.source.startsWith('JSON.stringify(')?'value':'evaluate'):op;
    const id=++this.next;return timeout(new Promise((resolve,reject)=>{this.pending.set(id,{resolve,reject});this.process.stdin.write(JSON.stringify({id,op,...extra})+'\n');}),15000,'native '+kind+' control deadline');
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

// Check supported application rules before starting any listener or browser.
try {
  const {stdout}=await execFileAsync('C:/Program Files/PowerShell/7/pwsh.exe',[
    '-NoLogo','-NoProfile','-NonInteractive','-File',path.join(repo,'support/ci/webrtc_permissions.ps1'),
    '-Kind','Headed','-Browser',args.get('--browser')||'both','-NativeHost',nativeExe,'-NodeProgram',process.execPath
  ],{encoding:'utf8',windowsHide:true,timeout:30000,maxBuffer:65536});
  assert(JSON.parse(stdout).applicationRuleCoverage===true,'application rules not confirmed');
} catch {
  throw Error('RTC application-rule preflight needs attention; no listeners or browsers started. Run support/ci/webrtc_permissions.ps1 -Kind Headed for details.');
}
const sourceAtStart={};for(const file of sourceFiles)sourceAtStart[file]=hash(await readFile(path.join(repo,file)));
const binaryAtStart=hash(await readFile(nativeExe));
const lockAtStart=hash(await readFile(path.join(repo,'Cargo.lock')));
const http=createServer((request,response)=>{response.writeHead(200,{'Content-Type':'text/html','Cache-Control':'no-store'});response.end('<!doctype html><title>RTC interoperability fixture</title><h1>RTC interoperability fixture</h1><p>Automated data-channel peer. No media capture.</p>');});
await new Promise(resolve=>http.listen(0,'127.0.0.1',resolve));
const url=`http://127.0.0.1:${http.address().port}/`;
const bind=await bindIP();
const sourceStableNow=async()=>{
  const hashes={};for(const file of sourceFiles)hashes[file]=hash(await readFile(path.join(repo,file)));
  return JSON.stringify(hashes)===JSON.stringify(sourceAtStart)&&binaryAtStart===hash(await readFile(nativeExe))&&lockAtStart===hash(await readFile(path.join(repo,'Cargo.lock')));
};
const journal=restart&&receiptMode==='append'?await RestartJournal.open(receipt,{
  advertisementRouting:localMdns?'Initial direct browser candidates delayed and SDP candidates omitted until connection, preventing native-initiated numeric fallback; native names untouched. Optional Firefox incoming aliases are limited to observed owned IPv4 sockets and do not qualify its responder.':undefined,
  gate,localMdns,restartDiagnostic,firefoxLog,firefoxIpc,mdnsObserveInterface,registrationProbes,dns:dnsEnabled,dnsMode,mdns:mdnsEnabled,mdnsAliasBrowser,sourceSHA256:sourceAtStart,nativeBinarySHA256:binaryAtStart,lockSHA256:lockAtStart,node:process.version,server,bind,expectedCases,answerDelayMs,
  choices:Object.fromEntries(dimensions.map(([flag,all])=>[flag.slice(2),choose(flag,all)]))
},execFileSync('git',['rev-parse','HEAD'],{cwd:repo,encoding:'utf8'}).trim(),args.get('--resume')):null;
if(journal)cases.push(...journal.cases());
const udpFixture=new UdpFixture();
const frontend=await udpFixture.start(bind);
const dnsFixture=dnsEnabled?new DnsFixture():null;
const dnsServer=dnsFixture?await dnsFixture.start(bind,dnsMode==='service'?Number(frontend.split(':')[1]):null):null;
const mdnsFixture=(mdnsEnabled||(localMdns&&mdnsAliasBrowser))?new MdnsFixture():null;
if(mdnsFixture)await mdnsFixture.start(bind,mdnsObserveInterface);
try {
  for(const browserName of choose('--browser',['chrome','firefox'])) {
    const browser=new Browser();
    const mapBrowserSignal=async signal=>{
      const alias=current?.browser===mdnsAliasBrowser;
      if(mdnsFixture&&alias)await mdnsFixture.prepareBrowser(signal,[browser.process.pid,browser.windowPid]);
      return mdnsFixture&&(!localMdns||alias)?mdnsFixture.rewrite(signal,alias):signal;
    };
    try {
      phase='browser startup';await browser.start(browserName,url);
      if(journal)await journal.browser(versions.at(-1));
      for(const engine of choose('--engine',['boa','nova']))for(const route of choose('--route',routes))
      for(const offerer of choose('--offerer',['native','browser']))for(const cleanup of choose('--cleanup',['close','navigate'])) {
        current={browser:browserName,engine,route,offerer,cleanup};
        if(journal?.has(current))continue;
        const native=new Native();const start=Date.now();let a,b,advertisement;
        const dnsBefore=dnsFixture?.snapshot();
        const mdnsBefore=mdnsFixture?.snapshot();
        try {
          browser.firefoxTrace?.clear();
          phase='peer startup';await browser.reset(url+'?case='+cases.length);native.start(engine,bind,dnsServer);await timeout(native.ready,45000,'native initialization deadline');
          const config={iceTransportPolicy:route==='relay'?'relay':'all',iceServers:route==='relay'?
            [{urls:[`turn:${frontend}?transport=udp`],username:'rtc-test',credential:'rtc-test-password'}]:route==='stun'?[{urls:[`stun:${frontend}`]}]:[]};
          const nativeConfig=JSON.parse(JSON.stringify(config));
          if(dnsEnabled&&route!=='direct')nativeConfig.iceServers[0].urls=dnsMode==='service'?[`${route==='relay'?'turn':'stun'}:rtc.example.test`]:nativeConfig.iceServers[0].urls.map(url=>url.replace(frontend,'rtc.example.test:'+frontend.split(':')[1]));
          await native.evaluate(source);await browser.evaluate(source);
          await native.evaluate(`globalThis.__interop=makeInteropPeer(${JSON.stringify(nativeConfig)},'native',${offerer==='native'});__interop.start()`);
          await browser.evaluate(`globalThis.__interop=makeInteropPeer(${JSON.stringify(config)},'browser',${offerer==='browser'});__interop.start()`);
          phase='negotiation and transfer';const exchangeStart=Date.now();
          const heldBrowserCandidates=[];
          for(let i=0;i<600;i++) {
            for(const signal of await native.value('__interop.takeSignals()')){if(localMdns){mdnsFixture?.watchNative(signal);if(mdnsFixture)browser.firefoxTrace?.watch(mdnsFixture.nativeNames);}await browser.evaluate(`__interop.receive(${JSON.stringify(signal)})`);}
            for(const signal of await browser.value('__interop.takeSignals()')){
              if(localMdns&&route==='direct'&&'candidate' in signal){assert(heldBrowserCandidates.length<128,'held candidate budget exceeded');heldBrowserCandidates.push(signal);continue;}
              const mapped=await mapBrowserSignal(signal);
              // Delayed trickling is intentional: require browser-initiated
              // connectivity to the native name before supplying a numeric route.
              if(localMdns&&route==='direct'&&mapped.description)mapped.description={...mapped.description,sdp:mapped.description.sdp.replace(/^a=candidate:.*\r?\n/gm,'')};
              await native.evaluate(`__interop.receive(${JSON.stringify(mapped)})`);
            }
            a=await native.value('__interop.snapshot()');b=await browser.value('__interop.snapshot()');
            assert(!udpFixture.error,udpFixture.error);
            assert(!a.errors.length && !b.errors.length,'peer error '+[...a.errors,...b.errors].join(','));
            if(a.connected && b.connected && a.received.length===3 && b.received.length===3)break;
            assert(Date.now()-exchangeStart<25000,'application exchange deadline');await delay(15);
          }
          verifySnapshot(a,'browser');verifySnapshot(b,'native');
          for(const signal of heldBrowserCandidates)await native.evaluate(`__interop.receive(${JSON.stringify(await mapBrowserSignal(signal))})`);
          phase='route and certificate verification';
          const initialStats=await settledStats(native,browser);
          let aStats=initialStats.a,bStats=initialStats.b;verifyStats(aStats,bStats,route);
          advertisement=localMdns?await inspectAdvertisement(native,bStats,route,bind):undefined;
          if(advertisement){advertisement.boundSocketQueryLines=(native.stderr.match(/RTC mdns selected socket query/g)||[]).length;advertisement.boundSocketQueryKinds=observedBoundQueryKinds(native.stderr);}
          if(advertisement&&browser.ipcTrace)advertisement.firefoxIpc=await browser.ipcTrace.snapshot();
          if(restartDiagnostic){const host=await native.value('__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type==="host").map(c=>({address:c.address,port:c.port}))[0]');if(host)advertisement.initialBrowserHosts=await browser.value(`__interop.hostEvidence(${JSON.stringify(host.address)},${JSON.stringify(bind)},${host.port})`);}
          if(dnsEnabled&&route!=='direct'){
            const local=await native.value('__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type!=="host").map(c=>({type:c.type,url:c.url,relayProtocol:c.relayProtocol,relatedAddress:c.relatedAddress,relatedPort:c.relatedPort,signalingKeys:Object.keys(c.toJSON())}))');
            assert(local.length>0&&local.every(c=>c.url===nativeConfig.iceServers[0].urls[0]),'original hostname URL metadata missing');
            assert(local.every(c=>!c.signalingKeys.includes('url')&&!c.signalingKeys.includes('relayProtocol')),'local metadata leaked into signaling');
            if(route==='relay')assert(local.every(c=>c.type==='relay'&&c.relayProtocol==='udp'&&c.relatedAddress==='0.0.0.0'&&c.relatedPort===0),'hostname relay privacy failed');
          }
          if(mdnsEnabled||localMdns){
            const deadline=Date.now()+8000;
            while((await native.request('metrics')).dnsPending&&Date.now()<deadline)await delay(25);
            assert((await native.request('metrics')).dnsPending===0,'initial name resolution did not settle');
            if(mdnsEnabled&&route!=='relay')assert((await native.value('__interop.pc.sctp.transport.iceTransport.getRemoteCandidates().filter(c=>c.address?.endsWith(".local")).length'))>=1,'signaled mDNS names were not preserved');
          }
          const initialSockets=restart?await ownedUdp(native.process.pid):null;
          let restarts;
          if(restart) {
            phase='ICE restart';restarts=await runRestartChecks({native,browser,udpFixture,offerer,route,verifySnapshot,verifyStats,answerDelayMs,mapBrowserSignal,restartDiagnostic,localBind:bind,requireMdns:mdnsEnabled,verifyAdvertisement:localMdns?(_a,b)=>inspectAdvertisement(native,b,route,bind,advertisement):null});
            a=await native.value('__interop.snapshot()');b=await browser.value('__interop.snapshot()');
            aStats=await native.value('__interop.stats');bStats=await browser.value('__interop.stats');verifyStats(aStats,bStats,route);
            phase='restart interruption';
            if(dnsFixture&&route==='stun')dnsFixture.pause=true;
            await native.evaluate('__interop.captureRestart();__interop.restart("method")');
            const deadline=Date.now()+15000;let pending;
            while(Date.now()<deadline){pending=await native.value('__interop.snapshot()');if(pending.signaling==='have-local-offer'&&pending.localCredentialsChanged)break;await delay(10);}
            assert(pending.signaling==='have-local-offer'&&pending.localCredentialsChanged,'interrupted restart did not apply an offer');
          }
          phase='native cleanup';
          const beforeMdns=await native.request('metrics');
          if(mdnsEnabled){
            assert(route==='relay'?beforeMdns.mdnsStarted===0:beforeMdns.mdnsResolved>=1,'missing mDNS resolution or relay suppression');
            const name=mdnsFixture.heldName();
            await native.evaluate(`__interop.pc.addIceCandidate({candidate:'candidate:held 1 udp 123456 ${name} 50000 typ host',sdpMLineIndex:0}).catch(e=>{throw e;})`);
            await delay(30);
          }
          if(localMdns){const deadline=Date.now()+8000;while((await native.request('metrics')).dnsPending&&Date.now()<deadline)await delay(25);}
          const metrics=await native.request('metrics'),dnsPending=metrics.dnsPending;
          const interruptedDns=dnsEnabled&&restart&&route==='stun';
          const interruptedMdns=mdnsEnabled&&route!=='relay';
          assert(dnsPending===Number(interruptedDns)+Number(interruptedMdns),'unexpected pending DNS ownership');
          const sockets=await ownedUdp(native.process.pid);
          // Windows' UDP owner table can omit shared 5353 bindings (confirmed
          // with an independent live bind). Count only observable endpoints;
          // provider socket/drop tests cover the receive-only membership.
          const sharedMdnsEndpointsObserved=sockets.filter(s=>s.port===5353).length;
          assert(sharedMdnsEndpointsObserved<=Number(interruptedMdns)+Number(localMdns&&route!=='relay')*2&&sockets.length===1+dnsPending+sharedMdnsEndpointsObserved,'unexpected observable RTC/DNS socket ownership');
          if(restart)assert(initialSockets.filter(s=>s.port!==5353).length===1&&sockets.some(s=>s.address===initialSockets.find(s=>s.port!==5353).address&&s.port===initialSockets.find(s=>s.port!==5353).port),'restart replaced the native RTC socket');
          let cleaned;
          if(cleanup==='navigate') {
            cleaned=await native.request('navigate');assert(cleaned.pending===0,'navigation retained a peer');
            assert(cleaned.stale[0]==='closed' && cleaned.stale.slice(1,3).every(x=>x==='closed') && cleaned.stale.slice(-2).every(x=>x==='InvalidStateError'),'navigation retained live RTC handles');
          } else {
            cleaned=await native.value('__interop.close()');assert(cleaned.state==='closed' && cleaned.channels.every(x=>x==='closed'),'close left live wrappers');
            assert((await native.request('metrics')).pending===0,'close retained a peer');
          }
          assert((await native.request('metrics')).dnsPending===0,'native cleanup retained DNS work');
          if(advertisement&&mdnsFixture){advertisement.observedIPv4Queries=mdnsFixture.nativeQueries-mdnsBefore.nativeQueries;advertisement.observedIPv4Answers=mdnsFixture.nativeAnswers-mdnsBefore.nativeAnswers;}
          if(advertisement){advertisement.observableShared5353=sharedMdnsEndpointsObserved;advertisement.exclusiveEndpointsRebound=sockets.filter(s=>s.port!==5353).length;}
          assert((await ownedUdp(native.process.pid)).length===0,'native cleanup retained UDP ownership');await released(sockets);
          await browser.evaluate('__interop.close()');
          const summary=(snapshot,stats)=>({connected:snapshot.connected,textMessagesVerified:2,binaryBytesVerified:48000,modes:snapshot.modes,refused:snapshot.refused,gatheringErrors:snapshot.gatheringErrors,
            pair:stats.pair,localFingerprint:stats.localFingerprint,remoteFingerprint:stats.remoteFingerprint,remoteDER_SHA256:stats.remoteDER_SHA256,statsRemoteFingerprint:stats.statsRemoteFingerprint,
            remoteAddressesRedacted:stats.remoteAddressesRedacted,peerReflexiveInRemoteList:stats.peerReflexiveInRemoteList});
          const dns=dnsFixture?{mode:dnsMode,queries:dnsFixture.queries-dnsBefore.queries,answers:dnsFixture.answers-dnsBefore.answers,held:dnsFixture.held-dnsBefore.held,types:Object.fromEntries(Object.entries(dnsFixture.types).map(([type,count])=>[type,count-dnsBefore.types[type]])),pendingAtCleanup:dnsPending,socketsReleased:sockets.length,sharedMdnsEndpointsObserved,originalUrlAndPrivacyChecked:route!=='direct'}:undefined;
          if(dns){assert(!dnsFixture.error,dnsFixture.error);assert(route==='direct'?dns.queries===0:dns.answers>=1,'missing controlled DNS exchange');assert(!a.gatheringErrors.length,'hostname gathering failed');if(interruptedDns)assert(dns.held>=1,'missing held DNS request at interruption');}
          if(dnsMode==='service'&&route!=='direct'){assert(dns.types.srv>=1&&dns.types.a>=1,'missing service lookup');assert(route==='relay'?dns.types.naptr===1:dns.types.naptr===0,'unexpected S-NAPTR ownership');}
          const mdns=mdnsFixture?{started:metrics.mdnsStarted,resolved:metrics.mdnsResolved,controlledNames:mdnsFixture.endpoints.size,browserNames:mdnsFixture.browserNames.size,aliasedBrowserNames:mdnsFixture.aliasedBrowserNames.size,answers:mdnsFixture.answers-mdnsBefore.answers,held:mdnsFixture.held-mdnsBefore.held,pendingAtCleanup:Number(interruptedMdns),relaySuppressed:route==='relay',namesAndStatsPrivate:true}:undefined;
          if(mdnsFixture)assert(!mdnsFixture.error,mdnsFixture.error);
          cases.push({...current,dns,mdns,advertisement,initialStatsSettling:initialStats.settling,status:'pass',seconds:Math.round((Date.now()-start)/10)/100,native:summary(a,aStats),browserPeer:summary(b,bStats),restarts,interruptedRestart:restart?true:undefined,socketPreserved:restart?true:undefined,cleanupVerified:true,socketRebound:true});
          if(journal)await journal.case(cases.at(-1),await sourceStableNow());
          console.log(JSON.stringify({...current,status:'pass',seconds:cases.at(-1).seconds}));
        } catch(error) {
          let resolution;try{const m=await native.request('metrics');resolution={started:m.mdnsStarted,resolved:m.mdnsResolved,pending:m.dnsPending,diagnostics:m.mdnsDiagnostics,fixture:mdnsFixture?.snapshot()};}catch{}
          if(resolution&&browser.ipcTrace){resolution.firefoxIpc=await browser.ipcTrace.snapshot();await delay(12000);resolution.firefoxIpcSettled=await browser.ipcTrace.snapshot();resolution.firefoxTransportSettled=browser.firefoxTrace.snapshot();}
          if(resolution){resolution.boundSocketQueryLines=(native.stderr.match(/RTC mdns selected socket query/g)||[]).length;resolution.boundSocketQueryKinds=observedBoundQueryKinds(native.stderr);}
          if(registrationProbes&&resolution){try{const host=await native.value('__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type==="host").map(c=>({address:c.address,port:c.port}))[0]');if(host)resolution.postFailureProbes=await probeRegistration(host.address,bind,mdnsFixture.socket);}catch{resolution.postFailureProbeError=true;}}
          if(registrationProbes&&resolution)resolution.afterProbeBoundSocketQueryKinds=observedBoundQueryKinds(native.stderr);
          try { await native.evaluate('__interop.inspect()'); await browser.evaluate('__interop.inspect()'); await delay(100); } catch {}
          let observedStats; try {observedStats={native:await native.value('__interop.stats'),browser:await browser.value('__interop.stats')};} catch {}
          for(const stats of Object.values(observedStats||{}))if(stats){stats.remoteDER_SHA256=(stats.remoteCertificates||[]).map(bytes=>hash(Buffer.from(bytes)));delete stats.remoteCertificates;}
          cases.push({...current,advertisement,status:'fail',phase,error:error.message.slice(0,180),resolution,observed:{native:a,browser:b},observedStats,initialStatsDiagnostics:error.initialStatsDiagnostics,restartDiagnostics:error.restartDiagnostics,trace:native.stderr.split('\n').filter(s=>s.startsWith('RTC poll variant:')).slice(-8)});
          if(journal)await journal.case(cases.at(-1),await sourceStableNow());
          if(!continueOnFailure)throw error;
          failure ||= {phase,error:error.message.slice(0,180)};process.exitCode=1;
          console.log(JSON.stringify({...current,status:'fail',phase,error:error.message.slice(0,180)}));
        } finally {
          try {await browser.evaluate('globalThis.__interop?.close()');}catch{}
          await native.close();if(dnsFixture)dnsFixture.pause=false;if(mdnsFixture)mdnsFixture.clear();await delay(100);await udpFixture.clear();
        }
      }
    } finally {await browser.close();if(journal&&browser.firefoxTrace)await journal.append('firefox-trace',{trace:browser.firefoxTrace.snapshot()});}
  }
} catch(error) {failure={phase,case:current,error:error.message.slice(0,180)};console.error(JSON.stringify(failure));process.exitCode=1;}
finally {
  if(dnsFixture)await dnsFixture.close();
  if(mdnsFixture)await mdnsFixture.close();
  await udpFixture.close();
  await new Promise(resolve=>http.close(resolve));
  const hashes={};for(const file of sourceFiles)hashes[file]=hash(await readFile(path.join(repo,file)));
  const sourceStable=JSON.stringify(hashes)===JSON.stringify(sourceAtStart)&&binaryAtStart===hash(await readFile(nativeExe))&&lockAtStart===hash(await readFile(path.join(repo,'Cargo.lock')));
  if(!sourceStable){failure={phase:'source identity',error:'source or fixture binary changed during qualification'};process.exitCode=1;}
  const record={schema:1,gate,sourceStable,lockSHA256:lockAtStart,utc:new Date().toISOString(),source:execFileSync('git',['rev-parse','HEAD'],{cwd:repo,encoding:'utf8'}).trim(),node:process.version,command:['node','support/ci/webrtc_interop.mjs',...process.argv.slice(2)],expectedCases,completeMatrix:expectedCases===(restart?48:32)&&cases.length===expectedCases&&!failure,nativeBinarySHA256:hash(await readFile(nativeExe)),sourceSHA256:hashes,versions,cases,failure,relayFixture:'bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 16 clients/256 pending datagrams/1 MiB',profileCache:scratch,profileCleanup:'earlier recursive cleanup was rejected as blocked by policy; marked profiles retained for reuse; reviewer not established',scope:localMdns?'Same-machine selected LAN-interface native IPv4 advertisement consumed by installed headed browsers; native UUID names untouched, direct browser named-host selection, privacy and lifecycle checked. Cross-machine, IPv6 and browser-native responder acceptance remain separate.':mdnsEnabled?'native Windows remote UUID mDNS resolution, controlled numeric-host aliases and untouched browser mDNS names, fixed-interface IPv4; relay policy suppression, restart and realm cleanup. Local name advertisement, IPv6 multicast, TURN TCP/TLS, public NAT, media and carriers remain open.':dnsMode==='service'?'native Windows headed restart with portless STUN/UDP-TURN and controlled SRV/S-NAPTR at nondefault discovered ports; browser peers use equivalent numeric endpoints. Fixed interface; mDNS, TURN TCP/TLS, public routing, media and carriers remain open.':dnsEnabled?'native Windows headed restart with explicit-port STUN/UDP-TURN hostnames and controlled A replies; browser peers use equivalent numeric endpoints. SRV/NAPTR, mDNS, TURN TCP/TLS, interface changes, media and carriers remain open.':restart?'native Windows headed data-channel restart on fixed numeric UDP servers':'native Windows headed data channels, controlled WSL UDP server'};
  if(journal) {
    await journal.finish(failure,sourceStable);
    console.log(`${gate} ${journal.passed.size}/${expectedCases} durable cases passed; run ${journal.run}`);
  } else if(receiptMode==='stdout') {
    console.log(`${gate} ${cases.filter(c=>c.status==='pass').length}/${cases.length} passed; receipt emitted`);
    console.log('RTC_RECEIPT '+JSON.stringify(record));
  } else {
  const old=await readFile(receipt,'utf8'),marker='\n\n## '+gate+' automated browser interoperability';
  const entry=marker+' - '+record.utc+'\n\n```json\n'+JSON.stringify(record)+'\n```\n';
  assert(Buffer.byteLength(old+entry)<=(gate==='E2ah'?340000:gate==='E2m'?244000:220000),gate+' receipt byte cap exceeded');
  await appendFile(receipt,entry);console.log(`${gate} ${cases.filter(c=>c.status==='pass').length}/${cases.length} passed; compact receipt appended`);
  }
}

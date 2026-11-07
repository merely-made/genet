"""Observe owned IPv4 mDNS names on Npcap adapters without opening UDP sockets."""
import ctypes as c
import ipaddress
import json
import os
import queue
import re
import struct
import sys
import threading
import time
from contextlib import ExitStack

GROUP = ipaddress.IPv4Address("224.0.0.251").packed
NAME = re.compile(r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.local")


def packet_fields(frame, link):
    if link == 1 and len(frame) >= 14 and frame[12:14] == b"\x08\x00":
        ip = frame[14:]
    elif link == 0 and frame[:4] == b"\x02\x00\x00\x00":
        ip = frame[4:]
    else:
        return None
    if len(ip) < 28 or ip[0] >> 4 != 4 or ip[9] != 17:
        return None
    header = (ip[0] & 15) * 4
    total, _, fragment = struct.unpack_from("!HHH", ip, 2)
    if header < 20 or header + 8 > total or total > len(ip) or fragment & 0x3fff:
        return None
    source, destination, length = struct.unpack_from("!HHH", ip, header)
    if 5353 not in (source, destination) or not 20 <= length <= min(1508, total - header):
        return None
    return ip[12:16], ip[16:20], source, destination, ip[8], ip[header+8:header+length]


def dns_events(packet, watched, address):
    ident, flags, questions, answers, authority, additional = struct.unpack_from("!6H", packet)
    if questions > 16 or answers + authority + additional > 32:
        return []
    response = bool(flags & 0x8000)
    events = []

    def name(offset, depth=0):
        if depth > 8:
            raise ValueError
        labels = []
        while True:
            length = packet[offset]
            offset += 1
            if not length:
                return b".".join(labels).lower(), offset
            if length & 0xc0 == 0xc0:
                target = ((length & 63) << 8) | packet[offset]
                labels.append(name(target, depth + 1)[0])
                return b".".join(labels).lower(), offset + 1
            if length > 63 or offset + length > len(packet) or len(labels) >= 32:
                raise ValueError
            labels.append(packet[offset:offset+length])
            offset += length

    offset = 12
    try:
        for _ in range(questions):
            key, offset = name(offset)
            kind, cls = struct.unpack_from("!HH", packet, offset)
            offset += 4
            if key in watched and not response:
                events.append({"kind": "query", "type": kind, "class": cls & 0x7fff,
                               "qu": bool(cls & 0x8000), "questions": questions})
        for section, count in (("answer", answers), ("authority", authority),
                               ("additional", additional)):
            for _ in range(count):
                key, offset = name(offset)
                kind, cls, ttl, length = struct.unpack_from("!HHIH", packet, offset)
                offset += 10
                end = offset + length
                if end > len(packet):
                    raise ValueError
                if key in watched and kind == 1 and length == 4:
                    events.append({"kind": "answer" if response else "query-record",
                                   "section": section, "ttl": ttl,
                                   "class": cls & 0x7fff, "flush": bool(cls & 0x8000),
                                   "selectedAddress": packet[offset:end] == address})
                offset = end
    except (IndexError, ValueError, struct.error):
        return []
    if offset != len(packet):
        return []
    for event in events:
        event.update(idZero=ident == 0, authoritative=bool(flags & 0x0400),
                     truncated=bool(flags & 0x0200))
    return events


class Header(c.Structure):
    _fields_ = [("seconds", c.c_long), ("micros", c.c_long),
                ("captured", c.c_uint32), ("length", c.c_uint32)]


class Filter(c.Structure):
    _fields_ = [("length", c.c_uint), ("instructions", c.c_void_p)]


class Stats(c.Structure):
    _fields_ = [(name, c.c_uint) for name in ("received", "dropped", "interface_dropped",
                 "captured", "sent", "network_dropped")]


class CaptureAddress(c.Structure):
    pass


CaptureAddress._fields_ = [("next", c.POINTER(CaptureAddress)),
                           ("address", c.c_void_p), ("mask", c.c_void_p),
                           ("broadcast", c.c_void_p), ("destination", c.c_void_p)]


class CaptureDevice(c.Structure):
    pass


CaptureDevice._fields_ = [("next", c.POINTER(CaptureDevice)), ("name", c.c_char_p),
                          ("description", c.c_char_p), ("addresses", c.POINTER(CaptureAddress)),
                          ("flags", c.c_uint)]


def select_devices(dll, addresses):
    devices = c.POINTER(CaptureDevice)()
    error = c.create_string_buffer(256)
    if dll.pcap_findalldevs(c.byref(devices), error):
        raise RuntimeError("mDNS capture adapter discovery failed")
    result = {}
    try:
        device = devices
        while device:
            address = device.contents.addresses
            while address:
                pointer = address.contents.address
                if pointer and c.c_ushort.from_address(pointer).value == 2:
                    ip = c.string_at(pointer + 4, 4)
                    if ip in addresses:
                        result[addresses[ip]] = device.contents.name
                address = address.contents.next
            device = device.contents.next
    finally:
        dll.pcap_freealldevs(devices)
    if len(result) != 2:
        raise RuntimeError("mDNS capture adapters unavailable for both addresses")
    return list(result.items()) + [("loopback", b"\\Device\\NPF_Loopback")]


def load_pcap():
    directory = os.path.join(os.environ["SystemRoot"], "System32", "Npcap")
    with os.add_dll_directory(directory):
        dll = c.CDLL(os.path.join(directory, "wpcap.dll"))
    specs = {
        "findalldevs": (c.c_int, [c.POINTER(c.POINTER(CaptureDevice)), c.c_char_p]),
        "freealldevs": (None, [c.POINTER(CaptureDevice)]),
        "open_live": (c.c_void_p, [c.c_char_p, c.c_int, c.c_int, c.c_int, c.c_char_p]),
        "compile": (c.c_int, [c.c_void_p, c.POINTER(Filter), c.c_char_p, c.c_int, c.c_uint32]),
        "setfilter": (c.c_int, [c.c_void_p, c.POINTER(Filter)]),
        "freecode": (None, [c.POINTER(Filter)]),
        "setnonblock": (c.c_int, [c.c_void_p, c.c_int, c.c_char_p]),
        "datalink": (c.c_int, [c.c_void_p]),
        "next_ex": (c.c_int, [c.c_void_p, c.POINTER(c.POINTER(Header)), c.POINTER(c.c_void_p)]),
        "stats": (c.c_int, [c.c_void_p, c.POINTER(Stats)]),
        "close": (None, [c.c_void_p]),
    }
    for name, (result, args) in specs.items():
        function = getattr(dll, "pcap_" + name)
        function.restype = result
        function.argtypes = args
    return dll


def run(selected_ip, alternate_ip):
    selected = ipaddress.IPv4Address(selected_ip).packed
    alternate = ipaddress.IPv4Address(alternate_ip).packed
    if selected == alternate:
        raise ValueError("capture adapters must differ")
    addresses = {selected: "selected", alternate: "alternate"}
    watched = set()
    messages = queue.Queue(maxsize=8)

    def read_commands():
        for line in sys.stdin:
            messages.put(json.loads(line))
        messages.put({"op": "stop"})

    def emit(value):
        print(json.dumps(value, separators=(",", ":")), flush=True)

    with ExitStack() as stack:
        dll = load_pcap()
        handles = []
        for role, device in select_devices(dll, addresses):
            error = c.create_string_buffer(256)
            handle = dll.pcap_open_live(device, 1600, 0, 10, error)
            if not handle:
                raise RuntimeError(f"cannot open {role} capture adapter")
            stack.callback(dll.pcap_close, handle)
            link = dll.pcap_datalink(handle)
            if link not in (0, 1):
                raise RuntimeError(f"unsupported {role} capture link type")
            program = Filter()
            if dll.pcap_compile(handle, c.byref(program), b"ip and udp and port 5353", 1, 0xffffffff):
                raise RuntimeError("cannot compile mDNS capture filter")
            try:
                if dll.pcap_setfilter(handle, c.byref(program)):
                    raise RuntimeError("cannot set mDNS capture filter")
            finally:
                dll.pcap_freecode(c.byref(program))
            if dll.pcap_setnonblock(handle, 1, error):
                raise RuntimeError("cannot poll mDNS capture adapter")
            handles.append((role, handle, link))

        threading.Thread(target=read_commands, daemon=True).start()
        start = time.time()
        deadline = time.monotonic() + 180
        events = []
        omitted = 0
        datagrams = {role: 0 for role, _, _ in handles}
        emit({"ready": True, "adapters": [role for role, _, _ in handles]})
        while time.monotonic() < deadline:
            for role, handle, link in handles:
                for _ in range(64):
                    header, data = c.POINTER(Header)(), c.c_void_p()
                    result = dll.pcap_next_ex(handle, c.byref(header), c.byref(data))
                    if result < 0:
                        raise RuntimeError(f"{role} capture stopped")
                    if result == 0:
                        break
                    datagrams[role] += 1
                    if not watched:
                        continue
                    fields = packet_fields(c.string_at(data, header.contents.captured), link)
                    if fields is None:
                        continue
                    source, destination, sport, dport, ttl, payload = fields
                    for event in dns_events(payload, watched, selected):
                        if len(events) >= 96:
                            omitted += 1
                            continue
                        timestamp = header.contents.seconds + header.contents.micros / 1e6
                        events.append(dict(event, ms=round((timestamp - start) * 1000),
                                           adapter=role, source=addresses.get(source, "other"),
                                           destination="multicast" if destination == GROUP else addresses.get(destination, "other"),
                                           source5353=sport == 5353, destination5353=dport == 5353,
                                           ipTtl=ttl))
            try:
                message = messages.get_nowait()
            except queue.Empty:
                time.sleep(0.01)
                continue
            match message["op"]:
                case "watch":
                    names = message["names"]
                    if len(names) > 8 or not all(NAME.fullmatch(name) for name in names):
                        raise ValueError("invalid mDNS capture names")
                    watched = {name.encode() for name in names}
                    emit({"watching": len(watched)})
                case "snapshot":
                    drops = {}
                    for role, handle, _ in handles:
                        stats = Stats()
                        drops[role] = None if dll.pcap_stats(handle, c.byref(stats)) else stats.dropped
                    emit({"ms": round((time.time() - start) * 1000),
                          "events": sorted(events, key=lambda event: event["ms"]),
                          "omitted": omitted, "captureDrops": drops,
                          "filteredDatagrams": datagrams})
                case "stop":
                    return
                case _:
                    raise ValueError("invalid capture command")
        raise RuntimeError("mDNS capture deadline")


if __name__ == "__main__":
    try:
        run(*sys.argv[1:3])
    except (ValueError, OSError, RuntimeError) as error:
        # Only the controller sees a failure status. Packet bytes never leave this process.
        print(json.dumps({"error": str(error) if isinstance(error, RuntimeError) else "capture setup failed"}), flush=True)
        sys.exit(1)

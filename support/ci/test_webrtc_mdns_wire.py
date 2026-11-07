"""Coverage for the mDNS packet reducer used by the optional Windows trace."""
import struct
import unittest

from webrtc_mdns_wire import GROUP, dns_events, packet_fields

NAME = b"53caef04-4218-4fb0-815d-f91a511bb8d2.local"
ADDRESS = b"\xc0\x00\x02\x01"
LABELS = b"".join(bytes([len(part)]) + part for part in NAME.split(b".")) + b"\0"


class WireTest(unittest.TestCase):
    def test_query_and_compressed_answer_have_no_endpoint_data(self):
        question = LABELS + struct.pack("!HH", 1, 1)
        query = struct.pack("!6H", 0, 0, 1, 0, 0, 0) + question
        answer = (struct.pack("!6H", 0, 0x8400, 1, 1, 0, 0) + question +
                  b"\xc0\x0c" + struct.pack("!HHIH", 1, 0x8001, 120, 4) + ADDRESS)
        query_event, = dns_events(query, {NAME}, ADDRESS)
        answer_event, = dns_events(answer, {NAME}, ADDRESS)
        self.assertEqual(query_event["kind"], "query")
        self.assertFalse(query_event["qu"])
        self.assertTrue(answer_event["selectedAddress"])
        self.assertEqual(answer_event["ttl"], 120)
        self.assertEqual(dns_events(answer[:-1], {NAME}, ADDRESS), [])
        self.assertEqual(dns_events(answer, set(), ADDRESS), [])
        self.assertNotIn(NAME.decode(), repr(answer_event))

    def test_packet_decoder_checks_capture_lengths_and_fragments(self):
        udp = struct.pack("!HHHH", 5353, 5353, 20, 0) + bytes(12)
        ip = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 40, 0, 0, 255, 17, 0,
                         ADDRESS, GROUP)
        frame = bytes(12) + b"\x08\0" + ip + udp
        self.assertEqual(packet_fields(frame, 1), (ADDRESS, GROUP, 5353, 5353, 255, bytes(12)))
        self.assertIsNone(packet_fields(frame[:-1], 1))
        loopback = b"\x02\0\0\0" + ip[:6] + b"\x20\0" + ip[8:] + udp
        self.assertIsNone(packet_fields(loopback, 0))


if __name__ == "__main__":
    unittest.main()

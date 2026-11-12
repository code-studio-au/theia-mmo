"""Real loopback traffic, configuration bounds and deterministic packet choices."""
import concurrent.futures
import socket
import threading
import time
import unittest

from network_profile import Profile
from udp_impairment import Relay


class Profiles(unittest.TestCase):
    def test_default_is_no_added_impairment_and_rtt_is_explicit(self):
        self.assertEqual(Profile().added_rtt_ms, 0)
        self.assertEqual(Profile(delay_ms=80).added_rtt_ms, 160)
        for direction in (0, 1):
            for ordinal in range(1, 100):
                self.assertEqual(Profile().delivery(direction, ordinal), 0)
                self.assertIsNone(Profile(loss_percent=100).delivery(direction, ordinal))

    def test_bounds_reject_invalid_configuration(self):
        for args in [dict(delay_ms=-1), dict(delay_ms=5001), dict(delay_ms=True),
                     dict(jitter_ms=5001), dict(seed=-1), dict(seed=2**32),
                     dict(loss_percent=float('nan')), dict(loss_percent=float('inf')),
                     dict(loss_percent=-1), dict(loss_percent=101), dict(loss_percent=True),
                     dict(outages=[]), dict(outages=((0, 0),)),
                     dict(outages=((10, 100), (50, 10))),
                     dict(outages=((3_600_000, 1),)), dict(outages=tuple((i, 1) for i in range(17)))]:
            with self.subTest(args=args), self.assertRaises(ValueError):
                Profile(**args)

    def test_seeded_jitter_and_loss_are_repeatable_per_direction(self):
        p = Profile(delay_ms=40, jitter_ms=60, loss_percent=10, seed=7)
        packets = [p.delivery(0, i) for i in range(1, 2001)]
        # Interleaving reverse traffic cannot change the forward sequence.
        for i in range(1, 700):
            p.delivery(1, i)
        self.assertEqual(packets, [p.delivery(0, i) for i in range(1, 2001)])
        drops = sum(d is None for d in packets)
        self.assertTrue(140 < drops < 260, drops)
        self.assertTrue(all(.04 <= d <= .1 for d in packets if d is not None))
        self.assertNotEqual(packets, [p.delivery(1, i) for i in range(1, 2001)])
        self.assertNotEqual(packets, [Profile(40, 60, 10, 8).delivery(0, i) for i in range(1, 2001)])

    def test_outage_boundaries_are_half_open(self):
        p = Profile(outages=((100, 100), (300, 100)))
        self.assertFalse(p.in_outage(.099))
        self.assertTrue(p.in_outage(.1))
        self.assertFalse(p.in_outage(.2))
        self.assertTrue(p.in_outage(.399))
        self.assertFalse(p.in_outage(.4))

    def test_two_real_connections_keep_independent_delay_and_payloads(self):
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server:
            server.bind(('127.0.0.1', 0))
            server.settimeout(.05)
            stop = threading.Event()

            def echo():
                while not stop.is_set():
                    try:
                        data, source = server.recvfrom(2048)
                        server.sendto(data, source)
                    except socket.timeout:
                        pass

            worker = threading.Thread(target=echo)
            worker.start()
            relays = [Relay(Profile()), Relay(Profile(delay_ms=80))]
            try:
                for relay in relays:
                    relay.start(server.getsockname())

                def exchange(index):
                    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
                        client.settimeout(3)
                        samples = []
                        for number in range(3):
                            payload = bytes([index, number]) + bytes(range(256)) * 4
                            start = time.monotonic()
                            client.sendto(payload, relays[index].address)
                            data, source = client.recvfrom(2048)
                            samples.append(time.monotonic() - start)
                            self.assertEqual(data, payload)
                            self.assertEqual(source, relays[index].address)
                        return samples

                with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
                    fast, slow = list(executor.map(exchange, (0, 1)))
                self.assertTrue(all(t >= .15 for t in slow), slow)
                # Wide scheduling allowance; only the configured link must wait.
                self.assertLess(min(fast), min(slow))
                for relay in relays:
                    relay.close()
                    self.assertEqual(relay.stats['received'], [3, 3])
                    self.assertEqual(relay.stats['forwarded'], [3, 3])
                    self.assertEqual(relay.stats['dropped'], 0)
                    self.assertEqual(relay.stats['overflow'], 0)
            finally:
                for relay in relays:
                    if relay.socket.fileno() != -1:
                        relay.close()
                stop.set()
                worker.join(timeout=2)
                self.assertFalse(worker.is_alive())

    def test_real_loss_and_queued_outage_then_recovery(self):
        # One delayed packet must be purged as the outage begins, never replayed.
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server, socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
            server.bind(('127.0.0.1', 0))
            server.settimeout(.6)
            relay = Relay(Profile(delay_ms=200, outages=((100, 200),)))
            try:
                relay.start(server.getsockname())
                client.sendto(b'before-outage', relay.address)
                with self.assertRaises(socket.timeout):
                    server.recvfrom(2048)
                self.assertEqual(relay.stats['outage_dropped'], 1)
                client.sendto(b'after-outage', relay.address)
                self.assertEqual(server.recvfrom(2048)[0], b'after-outage')
            finally:
                relay.close()
            relay = Relay(Profile(loss_percent=100))
            try:
                relay.start(server.getsockname())
                client.sendto(b'lost', relay.address)
                with self.assertRaises(socket.timeout):
                    server.recvfrom(2048)
            finally:
                relay.close()
            self.assertEqual(relay.stats['dropped'], 1)
            self.assertEqual(relay.stats['forwarded'], [0, 0])

    def test_legacy_fixture_keeps_original_loss_and_duplication(self):
        from collections import Counter
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server, socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
            server.bind(('127.0.0.1', 0))
            server.settimeout(2)
            relay = Relay()
            try:
                relay.start(server.getsockname())
                for number in range(1, 35):
                    client.sendto(bytes([number]), relay.address)
                expected = Counter({n: 2 if n % 17 == 0 else 1 for n in range(1, 35) if n % 10})
                received = Counter(server.recvfrom(2048)[0][0] for _ in range(sum(expected.values())))
                self.assertEqual(received, expected)
            finally:
                relay.close()
            self.assertEqual(relay.stats['dropped'], 3)
            self.assertEqual(relay.stats['duplicated'], 2)

    def test_delay_queue_remains_bounded_and_close_discards_pending_packets(self):
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server, socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
            server.bind(('127.0.0.1', 0))
            relay = Relay(Profile(delay_ms=5000))
            try:
                relay.start(server.getsockname())
                deadline = time.monotonic() + 3
                for batch in range(10):
                    for _ in range(60):
                        client.sendto(b'queued', relay.address)
                    while relay.stats['received'][0] < (batch + 1) * 60:
                        self.assertLess(time.monotonic(), deadline)
                        time.sleep(.001)
                self.assertEqual(relay.stats['high_water'], 512)
                self.assertEqual(relay.stats['overflow'], 88)
                self.assertEqual(relay.stats['forwarded'], [0, 0])
            finally:
                start = time.monotonic()
                relay.close()
                self.assertLess(time.monotonic() - start, 2)

    def test_lifecycle_refuses_remote_backend_and_restarts(self):
        relay = Relay(Profile())
        try:
            with self.assertRaises(ValueError):
                relay.start(('192.0.2.1', 1234))
            relay.start(('127.0.0.1', 1234))
            with self.assertRaises(ValueError):
                relay.start(('127.0.0.1', 1234))
        finally:
            relay.close()
        with self.assertRaises(ValueError):
            relay.start(('127.0.0.1', 1234))


if __name__ == '__main__':
    unittest.main()

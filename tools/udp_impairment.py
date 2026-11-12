"""Bounded loopback-only UDP relay for owned native game qualification."""
import heapq
import selectors
import socket
import threading
import time

from network_profile import Profile

QUEUE_LIMIT = 512


class Relay:
    def __init__(self, profile=None):
        # Preserve the historical qualification fixture for existing callers.
        # Editor sessions pass Profile() explicitly for no added impairment.
        if profile is not None and not isinstance(profile, Profile):
            raise TypeError('Expected a validated network Profile')
        self.profile = profile
        self.started_at = None
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(('127.0.0.1', 0))
        self.socket.setblocking(False)
        self.address = self.socket.getsockname()
        self.backend = None
        self.client = None
        self.stop = threading.Event()
        self.thread = None
        self.error = None
        self.stats = dict(received=[0, 0], forwarded=[0, 0], bytes=[0, 0],
                          dropped=0, outage_dropped=0, duplicated=0, reordered=0, high_water=0, overflow=0)

    def start(self, backend):
        if backend[0] != '127.0.0.1' or not 0 < backend[1] <= 65535:
            raise ValueError('Relay backend must be a loopback UDP endpoint')
        if self.thread is not None or self.stop.is_set():
            raise ValueError('Relay can only be started once')
        self.backend = backend
        self.started_at = time.monotonic()
        self.thread = threading.Thread(target=self._run, name='qualification-udp', daemon=True)
        self.thread.start()

    def _run(self):
        try:
            self._pump()
        except Exception as error:
            self.error = error

    def _pump(self):
        pending = []
        serial = 0
        last = [0, 0]
        with selectors.DefaultSelector() as selector:
            selector.register(self.socket, selectors.EVENT_READ)
            while not self.stop.is_set():
                selector.select(.003)
                outage = self.profile is not None and self.profile.in_outage(time.monotonic() - self.started_at)
                if outage:
                    self.stats['outage_dropped'] += len(pending)
                    self.stats['dropped'] += len(pending)
                    pending.clear()
                for _ in range(32):
                    try:
                        data, source = self.socket.recvfrom(2048)
                    except BlockingIOError:
                        break
                    direction = int(source == self.backend)
                    if not direction:
                        if self.client is None:
                            self.client = source
                        if source != self.client:
                            continue
                    destination = self.client if direction else self.backend
                    if destination is None:
                        continue
                    self.stats['received'][direction] += 1
                    self.stats['bytes'][direction] += len(data)
                    number = self.stats['received'][direction]
                    if outage:
                        self.stats['dropped'] += 1
                        self.stats['outage_dropped'] += 1
                        continue
                    if self.profile is None:
                        delay = None if number % 10 == 0 else .040 + ((number * 37) % 61) / 1000.
                        copies = 2 if number % 17 == 0 else 1
                    else:
                        delay = self.profile.delivery(direction, number)
                        copies = 1
                    if delay is None:
                        self.stats['dropped'] += 1
                        continue
                    self.stats['duplicated'] += copies - 1
                    for copy in range(copies):
                        if len(pending) >= QUEUE_LIMIT:
                            self.stats['overflow'] += 1
                            continue
                        serial += 1
                        due = time.monotonic() + delay + copy * .005
                        heapq.heappush(pending, (due, serial, data, destination, direction, number))
                    self.stats['high_water'] = max(self.stats['high_water'], len(pending))
                while pending and pending[0][0] <= time.monotonic():
                    _, _, data, destination, direction, number = heapq.heappop(pending)
                    self.socket.sendto(data, destination)
                    self.stats['forwarded'][direction] += 1
                    self.stats['reordered'] += int(number < last[direction])
                    last[direction] = max(last[direction], number)

    def close(self):
        self.stop.set()
        if self.thread is not None:
            self.thread.join(timeout=2)
            assert not self.thread.is_alive(), 'Relay failed to stop'
        self.socket.close()
        if self.error is not None:
            raise self.error

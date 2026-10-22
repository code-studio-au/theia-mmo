"""Bounded loopback-only UDP relay for owned native game qualification."""
import heapq
import selectors
import socket
import threading
import time

QUEUE_LIMIT = 512


class Relay:
    def __init__(self):
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
                          dropped=0, duplicated=0, reordered=0, high_water=0, overflow=0)

    def start(self, backend):
        assert backend[0] == '127.0.0.1' and backend[1] > 0
        self.backend = backend
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
                    if number % 10 == 0:
                        self.stats['dropped'] += 1
                        continue
                    copies = 2 if number % 17 == 0 else 1
                    self.stats['duplicated'] += copies - 1
                    for copy in range(copies):
                        if len(pending) >= QUEUE_LIMIT:
                            self.stats['overflow'] += 1
                            continue
                        serial += 1
                        due = time.monotonic() + (.040 + ((number * 37) % 61) / 1000. + copy * .005)
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

"""Bounded, repeatable packet impairment for local editor network playtests.

Delay is added in EACH direction. Jitter adds 0..jitter_ms per packet; measured
RTT remains a separate runtime measurement. This does not change simulation time.
"""
from dataclasses import dataclass
import hashlib
import math
import struct


@dataclass(frozen=True)
class Profile:
    delay_ms: int = 0
    jitter_ms: int = 0
    loss_percent: float = 0.
    seed: int = 0
    # (start, duration), relative to relay start, in milliseconds.
    outages: tuple[tuple[int, int], ...] = ()

    def __post_init__(self):
        for name, maximum in [('delay_ms', 5000), ('jitter_ms', 5000), ('seed', 2**32 - 1)]:
            value = getattr(self, name)
            if type(value) is not int or not 0 <= value <= maximum:
                raise ValueError(f'{name} must be an integer in 0..{maximum}')
        if (type(self.loss_percent) not in (int, float)
                or not 0 <= self.loss_percent <= 100 or not math.isfinite(self.loss_percent)):
            raise ValueError('loss_percent must be finite and in 0..100')
        if type(self.outages) is not tuple or len(self.outages) > 16:
            raise ValueError('outages must be a tuple of at most 16 windows')
        previous_end = 0
        for outage in self.outages:
            if type(outage) is not tuple or len(outage) != 2:
                raise ValueError('each outage must be (start_ms, duration_ms)')
            start, duration = outage
            if (type(start) is not int or type(duration) is not int
                    or start < previous_end or not 1 <= duration <= 60_000
                    or start + duration > 3_600_000):
                raise ValueError('outages must be ordered, nonoverlapping windows within one hour')
            previous_end = start + duration

    @property
    def added_rtt_ms(self):
        """Configured base RTT contribution, excluding jitter and real latency."""
        return self.delay_ms * 2

    def in_outage(self, elapsed_seconds):
        elapsed_ms = elapsed_seconds * 1000
        return any(start <= elapsed_ms < start + duration for start, duration in self.outages)

    def delivery(self, direction, ordinal):
        """Return an added delay in seconds, or None to drop this packet.

        Counter-based choices keep one direction independent of the other's packet
        count and thread scheduling. Reproducing choices still requires the same
        per-direction packet sequence; this is not deterministic game replay.
        """
        digest = hashlib.sha256(struct.pack('<IBQ', self.seed, direction, ordinal)).digest()
        drop, jitter = struct.unpack('<QQ', digest[:16])
        if drop < int(self.loss_percent / 100 * 2**64):
            return None
        return (self.delay_ms + jitter % (self.jitter_ms + 1)) / 1000

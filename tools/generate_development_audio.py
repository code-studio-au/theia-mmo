#!/usr/bin/env python3
"""Generate original quiet mono PCM cues; no recordings or third-party samples."""
import math
from pathlib import Path
import struct
import wave


def generate(output):
    output.mkdir(parents=True, exist_ok=True)
    for name, seconds, start, finish in [('cast', .12, 520., 780.), ('impact', .18, 360., 180.)]:
        rate = 16000
        frames = []
        for i in range(round(rate*seconds)):
            t = i/rate
            u = t/seconds
            envelope = math.sin(math.pi*u)**2 * math.exp(-3*u)
            phase = 2*math.pi*(start*t+(finish-start)*t*t/(2*seconds))
            sample = .18*envelope*(math.sin(phase)+.2*math.sin(2*phase))/1.2
            frames.append(struct.pack('<h', round(sample*32767)))
        with wave.open(str(output/f'{name}.wav'), 'wb') as wav:
            wav.setparams((1, 2, rate, len(frames), 'NONE', 'not compressed'))
            wav.writeframes(b''.join(frames))


if __name__ == '__main__':
    generate(Path('content/gameplay/audio'))

"""Owned PulseAudio/PipeWire null-sink capture; never reads a desktop/mic source."""
import array
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def pulse(*args):
    return subprocess.check_output(['pactl', *args], text=True).strip()


class Capture:
    def __init__(self, output):
        self.output = Path(output)
        self.name = f'theia_qualification_{os.getpid()}_{time.monotonic_ns()}'
        self.module = None
        self.recorder = None
        self.log = None
        self.default = pulse('get-default-sink')
        self.foreign = []
        self.sink_index = None

    def start(self):
        try:
            self.module = pulse('load-module', 'module-null-sink', f'sink_name={self.name}',
                                'rate=48000', 'channels=2', 'sink_properties=device.description=TheiaQualification')
            assert self.module.isdigit(), 'Unexpected module identifier'
            assert pulse('get-default-sink') == self.default, 'Null sink changed the default; capture not started'
            deadline = time.monotonic()+5
            while time.monotonic() < deadline:
                sinks = json.loads(pulse('-f', 'json', 'list', 'sinks'))
                found = next((s for s in sinks if s['name'] == self.name), None)
                if found:
                    self.sink_index = found['index']
                    break
                time.sleep(.02)
            assert self.sink_index is not None, 'Owned sink did not appear'
            pulse('set-sink-mute', self.name, '0')
            pulse('set-sink-volume', self.name, '100%')
            self.audit([])
            config = self.output/'alsa-private.conf'
            config.write_text('pcm.!default { type pulse }\nctl.!default { type pulse }\n')
            self.log = (self.output/'audio-capture.log').open('w')
            self.recorder = subprocess.Popen([
                'parec', f'--device={self.name}.monitor', '--file-format=wav', '--format=s16le',
                '--rate=48000', '--channels=2', str(self.output/'feedback.wav'),
            ], stdout=self.log, stderr=subprocess.STDOUT)
            return {'PULSE_SINK': self.name, 'ALSA_CONFIG_PATH': str(config),
                    'PULSE_PROP': 'application.name=TheiaQualification application.id=au.theia.qualification'}
        except BaseException:
            self.close()
            raise

    def audit(self, owned_pids):
        inputs = json.loads(pulse('-f', 'json', 'list', 'sink-inputs'))
        for item in inputs:
            if item['sink'] != self.sink_index:
                continue
            pid = int(item.get('properties', {}).get('application.process.id', '-1'))
            if pid not in owned_pids:
                self.foreign.append({'input': item['index'], 'pid': pid})
                raise AssertionError('Unowned stream entered the isolated sink; stopping capture')
        assert pulse('get-default-sink') == self.default, 'Default sink changed during qualification'

    def close(self):
        try:
            if self.recorder and self.recorder.poll() is None:
                self.recorder.send_signal(signal.SIGINT)
                try:
                    self.recorder.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.recorder.kill()
                    self.recorder.wait()
            if self.log:
                self.log.close()
        finally:
            if self.module is not None:
                subprocess.run(['pactl', 'unload-module', self.module], check=True, stdout=subprocess.DEVNULL)
                self.module = None

    def verify(self):
        import wave
        with wave.open(str(self.output/'feedback.wav'), 'rb') as wav:
            assert wav.getnchannels() == 2 and wav.getsampwidth() == 2 and wav.getframerate() == 48000
            assert wav.getnframes() <= 48000*360
            samples = array.array('h', wav.readframes(wav.getnframes()))
        if sys.byteorder != 'little':
            samples.byteswap()
        assert samples, 'No PCM captured from owned sink'
        peak = max(abs(v) for v in samples)/32768
        rms = math.sqrt(sum(v*v for v in samples)/len(samples))/32768
        nonzero = sum(v != 0 for v in samples)
        assert rms > .00001 and nonzero > 1000, f'No actual cue signal: {rms=}, {nonzero=}'
        assert peak < .99, f'Clipped feedback mix: {peak=}'
        unchanged = pulse('get-default-sink') == self.default
        assert unchanged and not self.foreign
        report = dict(passed=True, source=f'{self.name}.monitor', microphone=False, desktop_monitor=False,
                      default_sink_unchanged=unchanged, foreign_inputs=self.foreign, sample_rate=48000,
                      channels=2, samples=len(samples), rms=rms, peak=peak, nonzero_samples=nonzero,
                      scope='Only the owned null sink; game child streams routed explicitly; not physical-speaker/HRTF quality')
        (self.output/'audio-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        return report

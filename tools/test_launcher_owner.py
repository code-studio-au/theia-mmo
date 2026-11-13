"""Real owner pipes and interpreter shutdown; no game or account services."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import unittest

from launcher_owner import watch_owner


class OwnerPipeTests(unittest.TestCase):
    def test_data_keeps_owner_alive_until_pipe_eof(self):
        read_fd, write_fd = os.pipe()
        stopped = threading.Event()
        with os.fdopen(read_fd, 'rb') as stream:
            worker = threading.Thread(target=watch_owner, args=(stream, stopped), daemon=True)
            worker.start()
            try:
                os.write(write_fd, b'owner remains alive')
                self.assertFalse(stopped.wait(.1))
            finally:
                os.close(write_fd)
            worker.join(2)
            self.assertFalse(worker.is_alive())
            self.assertTrue(stopped.is_set())

    @unittest.skipUnless(os.name == 'posix', 'POSIX SIGINT; Windows native shutdown is separate')
    def test_sigint_exits_cleanly_with_owner_pipe_still_open(self):
        source = '''import signal, sys, threading, time
from launcher_owner import watch_owner
stopped = threading.Event()
threading.Thread(target=watch_owner, args=(sys.stdin.buffer, stopped), daemon=True).start()
time.sleep(.15)
print('ready', flush=True)
try:
    while not stopped.wait(.1):
        pass
except KeyboardInterrupt:
    pass
'''
        process = subprocess.Popen([sys.executable, '-c', source], cwd=Path(__file__).parent,
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, start_new_session=True)
        try:
            self.assertEqual(process.stdout.readline().strip(), 'ready')
            process.send_signal(signal.SIGINT)
            # Keep stdin open throughout shutdown, as an editor that has not
            # closed its lifetime pipe would. communicate() would hide the bug.
            result = process.wait(timeout=8)
            errors = process.stderr.read()
            self.assertEqual(result, 0, errors)
            self.assertNotIn('Fatal Python error', errors)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            process.stdin.close()
            process.stdout.close()
            process.stderr.close()


if __name__ == '__main__':
    unittest.main()

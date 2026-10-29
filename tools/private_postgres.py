"""Owned loopback PostgreSQL test cluster; no system service or user database."""
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import time


class Cluster:
    def __init__(self, output, sdk):
        self.output = Path(output).resolve()
        self.sdk = Path(sdk).resolve()
        self.bin = self.sdk/'usr/lib/postgresql/16/bin'
        self.server = None
        self.log = None
        self.env = dict(os.environ)
        self.env['LD_LIBRARY_PATH'] = str(self.sdk/'usr/lib/x86_64-linux-gnu')+':'+self.env.get('LD_LIBRARY_PATH', '')
        self.env.pop('PGPASSWORD', None)
        self.password = secrets.token_urlsafe(32)
        self.port = None

    def __enter__(self):
        self.output.mkdir(mode=0o700, parents=True, exist_ok=False)
        try:
            (self.output/"theia-owned-test-cluster").write_text("Theia disposable PostgreSQL fixture v1\n")
            password = self.output/'password'
            password.write_text(self.password+'\n')
            password.chmod(0o600)
            with socket.socket() as reserved:
                reserved.bind(('127.0.0.1', 0))
                self.port = reserved.getsockname()[1]
            pgpass = self.output/'pgpass'
            pgpass.write_text(f'127.0.0.1:{self.port}:postgres:theia_test:{self.password}\n')
            pgpass.chmod(0o600)
            self.env['PGPASSFILE'] = str(pgpass)
            with (self.output/'initdb.log').open('w') as log:
                subprocess.run([str(self.bin/'initdb'), '-D', str(self.output/'data'),
                                '-L', str(self.sdk/'usr/share/postgresql/16'), '--username=theia_test',
                                '--auth=scram-sha-256', '--pwfile='+str(password), '--locale=C',
                                '--encoding=UTF8', '--data-checksums'], env=self.env, stdout=log,
                               stderr=subprocess.STDOUT, check=True, timeout=30)
            password.unlink()
            self.log = (self.output/'server.log').open('w')
            self.start()
            return self
        except BaseException:
            self.close()
            raise

    def start(self):
        assert self.server is None
        self.server = subprocess.Popen([str(self.bin/'postgres'), '-D', str(self.output/'data'),
                                        '-h', '127.0.0.1', '-p', str(self.port), '-k', str(self.output),
                                        '-c', 'shared_buffers=16MB', '-c', 'max_connections=16',
                                        '-c', 'fsync=on', '-c', 'synchronous_commit=on',
                                        '-c', 'full_page_writes=on'], env=self.env,
                                       stdout=self.log, stderr=subprocess.STDOUT)
        deadline = time.monotonic()+20
        while time.monotonic() < deadline:
            assert self.server.poll() is None, 'Owned PostgreSQL exited; inspect server.log'
            if self.query('SELECT 1;').returncode == 0:
                return
            time.sleep(.05)
        raise TimeoutError('Owned PostgreSQL startup timeout')

    def query(self, sql, *, wrong_password=False):
        env = dict(self.env, PGPASSWORD='synthetic-negative-test') if wrong_password else self.env
        return subprocess.run([str(self.bin/'psql'), '-X', '-w', '-A', '-t', '-v', 'ON_ERROR_STOP=1',
                               '-h', '127.0.0.1', '-p', str(self.port), '-U', 'theia_test', '-d', 'postgres'],
                              input=sql, text=True, capture_output=True, env=env, timeout=5)

    def test_environment(self):
        # Pass in the child environment only. Never print/serialize this mapping.
        return dict(self.env, THEIA_TEST_POSTGRES_DATA=str(self.output/'data'), THEIA_TEST_DATABASE_URL=f'postgresql://theia_test:{self.password}@127.0.0.1:{self.port}/postgres')

    def stop(self):
        if self.server is not None and self.server.poll() is None:
            self.server.send_signal(signal.SIGINT)
            try:
                self.server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.server.kill()
                self.server.wait()
        self.server = None

    def restart(self):
        self.stop()
        self.start()

    def close(self):
        try:
            self.stop()
        finally:
            if self.log:
                self.log.close()
            for name in ['password', 'pgpass']:
                (self.output/name).unlink(missing_ok=True)

    def __exit__(self, *_):
        self.close()

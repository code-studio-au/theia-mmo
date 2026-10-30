#!/usr/bin/env python3
"""Actual verified OIDC, encrypted ticket and PostgreSQL lease retirement proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
from private_identity import Identity
from private_postgres import Cluster


def run(output,identity_sdk,postgres_sdk,inventory=False,rewards=False,progression=False,quests=False,vendors=False):
    progression = progression or quests
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    with (output/'build.log').open('w') as log:
        subprocess.run(['cargo','build','--locked','-p','theia-server','--example','admission_lease_qualification','-j2'],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=600)
    binary=Path('target/debug/examples/admission_lease_qualification').resolve()
    child=None
    with Identity(output/'provider',identity_sdk) as identity, Cluster(output/'cluster',postgres_sdk) as cluster:
        env=dict(cluster.test_environment(),THEIA_TEST_OIDC_ISSUER=identity.issuer,THEIA_TEST_OIDC_CLIENT=identity.client_id,THEIA_TEST_OIDC_REDIRECT=identity.redirect,THEIA_TEST_VENDORS="1" if vendors else "0",THEIA_TEST_INVENTORY="1" if inventory else "0",THEIA_TEST_REWARDS="1" if rewards else "0",THEIA_TEST_PROGRESSION="1" if progression else "0",THEIA_TEST_QUESTS="1" if quests else "0")
        with (output/'rust-errors.log').open('w') as log:
            try:
                child=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log,env=env,bufsize=0)
                pending=b'';result=None
                with selectors.DefaultSelector() as selector:
                    selector.register(child.stdout,selectors.EVENT_READ)
                    while result is None:
                        while b'\n' not in pending:
                            assert selector.select(timeout=40),'Owned lease contract timed out'
                            chunk=os.read(child.stdout.fileno(),16384)
                            assert chunk,'Lease contract ended before its summary'
                            pending+=chunk;assert len(pending)<=32768
                        line,pending=pending.split(b'\n',1);message=json.loads(line)
                        if 'authorization_url' in message:
                            callback=identity.authorize(message['authorization_url'])
                            child.stdin.write((json.dumps(callback)+'\n').encode());child.stdin.flush()
                        else:
                            assert message['passed']
                            if vendors:assert message['vendor_enabled'] and message['accepted_executing_and_queued_purchases_survive_logout']
                            if quests:assert message['objective_enabled'] and message['accepted_objective_facts_survive_retirement']
                            result=message
                assert child.wait(timeout=10)==0
            finally:
                if child is not None:
                    if child.poll() is None:
                        child.terminate()
                        try:child.wait(timeout=10)
                        except subprocess.TimeoutExpired:child.kill();child.wait()
                    child.stdin.close();child.stdout.close()
        text=(output/'rust-errors.log').read_text()
        assert identity.password not in text and cluster.password not in text
    result.update(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),owned_services_stopped=identity.server is None and cluster.server is None)
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--vendors',action='store_true',help='Exercise accepted queued and executing barter retirement')
    parser.add_argument('--quests',action='store_true',help='Exercise coupled XP/objective retirement')
    parser.add_argument('--progression',action='store_true',help='Exercise accepted XP facts and retirement')
    parser.add_argument('--rewards',action='store_true',help='Exercise accepted reward offers and claim retirement')
    parser.add_argument('--inventory',action='store_true',help='Exercise actual item-owner retirement and backend-loss policy')
    parser.add_argument('--identity-sdk',type=Path,default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk',type=Path,default=Path('build/postgres-sdk'))
    args=parser.parse_args()
    print(json.dumps(run(args.output,args.identity_sdk.resolve(),args.postgres_sdk.resolve(),inventory=args.inventory,rewards=args.rewards,progression=args.progression,quests=args.quests,vendors=args.vendors),indent=2))

# Sydney development realm lifecycle

P5-23 / P6-09. This adds on-demand Play to the existing Cognito launcher,
hosted account service and encrypted Renet connection. The Lambda does no
gameplay work. The server remains the authority for admission and occupancy.

## Build and review

Use Rust 1.99, Node 26, the Rust x86_64 Linux musl target, musl-gcc and zip.

```sh
cargo test --locked --manifest-path infrastructure/gateway/Cargo.toml
python3 -m unittest discover -s tools -p test_realm_gateway.py
bash infrastructure/gateway/package.sh
(cd infrastructure/aws && npm ci --ignore-scripts && npm run check && npm test &&
  npm run synth -- --app 'node bin/theia-realm-gateway.ts' --quiet)
```

The package builds a static AL2023 Lambda bootstrap and the Linux host
controller. The CDK template can be synthesized without AWS credentials. The
workflow uploads the artifacts and the template even when deployment is disabled.

## AWS rollout

Reuse the Sydney EC2 instance and the existing Cognito tester pool and app
client. Keep the existing Cognito scope and the HTTPS and Renet listeners.

An AWS administrator first creates the separate
`TheiaRealmGatewayIdentity` stack. The existing host deployer remains scoped to
`TheiaDevelopmentHost`. The identity stack owns the gateway roles and a single
Sydney HTTP API without any routes or stage. This lets the gateway CloudFormation
role manage authorizers, integrations, routes and stages on exactly that API.
The CloudFormation role can pass only the gateway Lambda runtime role and cannot
edit IAM policies, delete the realm table or start EC2. Only the runtime role
can start the designated instance. The identity stack also attaches the single
realm-item policy to the existing EC2 instance role.

Synthesize and prepare an identity change set from `infrastructure/aws`.
Use the reviewed EC2 instance ID and an existing same-account, private, versioned
artifact bucket in Sydney with SSE-S3 encryption. These scoped roles do not
grant access to a customer-managed KMS key.

```sh
npm run synth -- --app 'node bin/theia-realm-gateway-identity.ts' \
  --output cdk.out/realm-identity --quiet
aws cloudformation deploy --region ap-southeast-2 \
  --stack-name TheiaRealmGatewayIdentity \
  --template-file cdk.out/realm-identity/TheiaRealmGatewayIdentity.template.json \
  --capabilities CAPABILITY_NAMED_IAM --no-execute-changeset \
  --parameter-overrides "InstanceId=$THEIA_INSTANCE_ID" "ArtifactBucket=$THEIA_GATEWAY_BUCKET"
```

Inspect the change set before executing it with the administrator identity.
Copy `DeploymentRoleArn` and `ExecutionRoleArn` from the identity stack outputs
to `THEIA_GATEWAY_DEPLOY_ROLE` and `THEIA_GATEWAY_EXECUTION_ROLE` in the protected
`theia-infrastructure` GitHub environment. Also set `THEIA_AWS_ACCOUNT_ID`,
`THEIA_GATEWAY_BUCKET` (the same reviewed bucket), `THEIA_GATEWAY_COGNITO_ISSUER`
and `THEIA_GATEWAY_COGNITO_CLIENT_ID`. The workflow scopes uploads to
`theia/realm-gateway/*.zip` and passes only the gateway CloudFormation role.

Dispatch **Theia realm gateway** on main in `prepare` mode. Review the printed
change set and artifact digest, then dispatch `execute` with the full change-set ARN.
This creates the Lambda, on-demand realm table and JWT-protected routes on the
approved API. The application stack imports the API, instance, bucket and
runtime role from the identity stack. Keep the identity stack and its exports
while the gateway stack uses them. Record the GatewayOrigin and RealmTable
outputs and keep the prior artifact version for rollback.

## Host and launcher

1. Start the existing host manually for controlled rollout. Keep the old launcher
   config and the active server release for rollback. Stage the lifecycle-aware
   server in a separate release. From the exact deployed release, record
   `server`, `launcher_min`, `client`, `protocol` and `content` in
   `/opt/theia/current/realm-release.json`. Use the released client version,
   the matched protocol identifier, and the packaged world/content identity.
   Point `THEIA_REALM_RELEASE_FILE` at the actual active release if it uses
   another path. Seed the table with the same record using the commands below
   while in maintenance. Keep the realm in maintenance during activation.
2. Install the verified `theia-realm-host` binary under `/opt/theia/bin/`.
   Install the three files in `systemd/` as the controller unit, a drop-in on
   `theia-development.service`, and the tmpfiles entry, respectively. Match the
   actual unprivileged game service user/group in tmpfiles. The runtime directory
   is root-owned; only its game observation subdirectory is writable by the game.
   Apply tmpfiles and reload
   systemd, then start the matched server and enable the controller. Keep the
   existing PostgreSQL data and start ordering. The instance's shutdown behavior
   must remain `stop`.
3. Remove maintenance once the live server and the controller report a fresh,
   compatible heartbeat. Package a new launcher version and add the optional
   `gateway` object to its existing trusted realm config. Use
   `{"origin":"<GatewayOrigin>","protocol":"<deployed protocol>","content":"<deployed content>"}`.
   Use the gateway's Sydney execute-api HTTPS origin. The launcher verifies
   installed files before asking the gateway to start the instance, then opens
   the native roster through the existing Cognito realm exchange. A copy of the
   config without the gateway object still supports the old hosted login path
   when the server's lifecycle drop-in is disabled.
4. Confirm unauthenticated and non-tester requests cannot reserve or start.
   Stop the instance, then have both testers click Play, wait for real readiness
   and enter Millhaven. Keep one tester in the roster to exercise lease renewal,
   and verify a pending reservation prevents idle stop. Disconnect, confirm a
   successful drain and instance stop after five idle minutes, then wake the
   realm and confirm both characters' progress. Rehearse maintenance and a
   withheld flush marker on an isolated release before relying on automatic stop.

DynamoDB holds one realm item, updated with strongly consistent reads and
version-checked writes. At most eight join reservations live inside the item.
The host removes a reservation on authoritative admission; launcher cancellation
removes its own reservation, and heartbeats prune expired entries. Idle shutdown
requires an empty reservation map. The same item retains release compatibility
and lifecycle metadata while the instance is stopped.
The game drops locally cached joins absent from the current valid lease snapshot
when a tester signs in. An admission reports the reservation captured when its
character ticket was issued, even if that account has since requested another
join.

While EC2 runs, the controller reads the small item roughly every four seconds
to deliver new reservations to the game. It writes on a change or when its last
heartbeat is at least 15 seconds old. An unchanged steady state takes roughly
900 reads and 225 writes per running hour, excluding tester requests and retries.
An observation older than 40 seconds cannot advertise readiness. The launcher
polls only during Play and renews a reservation only until admission or exit.
The host does not poll while stopped.
A wake request after a committed drain waits for the next EC2 boot.
The host verifies the active game and join reservations before taking the
shutdown claim. A local admission that races that claim is fenced by the
server's admission lock. The host powers off only after the server's
run-specific flush marker. A missing marker leaves the instance available
for operator recovery and records an error. EC2 compute stops at power-off;
EBS, public IPv4 and service requests can still incur charges.

## Operator commands and rollback

Supply `AWS_REGION=ap-southeast-2` and `THEIA_REALM_TABLE=TheiaDevelopmentRealm`.
Use the instance role over SSM on the running host or a scoped operator IAM
session off-host. For operations while the instance is stopped, also supply
`THEIA_INSTANCE_ID` and allow DescribeInstances so the controller CLI can
reconcile the last drain marker without waking the host.

```sh
theia-realm-host status
theia-realm-host maintenance on
theia-realm-host activate /opt/theia/current/realm-release.json
theia-realm-host maintenance off
theia-realm-host keep-alive 30
```

Keep-alive accepts 30, 60, 120 or 240 minutes, or 0 to clear. In the absence of a
flush marker, inspect the owned server with SSM and verify PostgreSQL before
restarting the game. Enter maintenance and use `clear-error` after repair.
An instance that never reports ready may need manual recovery. The controller
does not power off a game whose persistence state it cannot verify.

For rollback, put the gateway in maintenance and wait for players to disconnect.
Disable the controller and remove the game lifecycle drop-in, reload systemd and
restart the previous compatible game release. Restore the launcher config without
the gateway object. Retain the table and artifact history until the rollback is
verified. The cloud workflow does not install software on EC2 or change the
previous game release.

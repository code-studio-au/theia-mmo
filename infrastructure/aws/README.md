# Theia AWS development host

This is the first P5-23 hosting slice. It defines one Ubuntu 24.04 LTS AMD64
`t3a.medium` in Sydney. It has an encrypted 40 GiB gp3 root volume, an Elastic
IPv4 address and a dedicated subnet, security group and instance role. There
are no inbound rules. Use Systems Manager after the instance registers with
the agent included in the Canonical AWS image. The host has outbound HTTPS.
Use HTTPS package sources for host preparation.

## Hosted development authentication

Synthesize the bootstrapless `TheiaDevelopmentCognito` stack to provision
managed browser login. The pool permits administrator-created accounts. The
public launcher client uses authorization code with PKCE and the registered
desktop callback. Add enrolled testers to `theia-testers`. The game server
verifies the access token's signature, issuer, client, expiry, group and scope.

```sh
npm exec -- cdk synth --app 'node bin/theia-cognito.ts' \
  --no-lookups --output cdk.out/cognito --quiet
npm exec -- cdk synth --app 'node bin/theia-realm-ingress.ts' \
  --no-lookups --output cdk.out/realm-ingress --quiet
```

The approved `TheiaDevelopmentRealmIngress` stack adds TCP 443 and UDP 5000
to the game's security group after the host TLS and admission checks pass.
Removing the ingress stack closes tester access. Store the CA signing key and
tester credentials privately. The launcher configuration carries the CA certificate.

The stack enables termination protection on the stack and on the instance.
The root volume survives stop/start and is retained on instance termination.
A replacement instance starts with a new root volume. Take and verify a
PostgreSQL dump before a replacement or schema change, then use an explicit
restore. A retained volume keeps accruing storage charges until it is removed.
The Elastic IP also incurs charges while the host is stopped. Standard CPU
credits avoid surplus CPU charges. Measure tick latency after credits are low
to see whether the instance can sustain two players.

This stack prepares the host for Systems Manager. PostgreSQL installation,
the game systemd unit, release deployment, Cognito and the wake/idle controller
are separate steps. The game and account endpoints remain closed until remote
admission and TLS have been qualified.

## Validate the template

With Node 26:

```sh
cd infrastructure/aws
npm ci --ignore-scripts
npm run check
npm test
npm run synth -- --quiet
```

The synthesized template is `cdk.out/TheiaDevelopmentHost.template.json`.
Synthesis runs with context lookups disabled and needs no AWS session. GitHub
Actions runs the same validation for infrastructure changes.

## Read-only Sydney preflight

```sh
aws sts get-caller-identity --region ap-southeast-2
aws cloudformation describe-stacks --stack-name CDKToolkit --region ap-southeast-2 --query 'Stacks[0].Outputs'
aws ssm get-parameter --region ap-southeast-2 --name /cdk-bootstrap/hnb659fds/version --query Parameter.Value
aws ssm get-parameter --region ap-southeast-2 \
  --name /aws/service/canonical/ubuntu/server/noble/stable/current/amd64/hvm/ebs-gp3/ami-id \
  --query Parameter.Value
aws ec2 describe-instance-type-offerings --region ap-southeast-2 \
  --location-type availability-zone --filters Name=instance-type,Values=t3a.medium
aws cloudformation validate-template --region ap-southeast-2 \
  --template-body file://cdk.out/TheiaDevelopmentHost.template.json
```

Verify the AMI with `describe-images --owners 099720109477`, including AMD64,
EBS and root mapping `/dev/sda1`. Pass the reviewed image ID and availability
zone as parameters so ordinary stack updates preserve the selected image and
zone. The default CDK synthesizer targets the existing `hnb659fds` bootstrap.
Review the execution roles and bucket settings before deploying. Keep any change
to shared bootstrap resources under separate review.

Read-only preflight on 24 November 2026 found CDKToolkit v32, a private
KMS-encrypted bootstrap bucket, and the Canonical image
`ami-048c1a9eb63a67eed` (Ubuntu Noble AMD64, `/dev/sda1`). All three Sydney
availability zones offered `t3a.medium`. CloudFormation validated the template
and the template-only diff showed a new stack. The existing CloudFormation
execution role has `AdministratorAccess`; review the deployment identity and
scope before granting an automated workflow access to it.

The separate `TheiaDeploymentIdentity` stack defines two roles under
`/theia/`. `TheiaDevelopmentDeployer` can prepare and execute changes to
`TheiaDevelopmentHost` and pass only `TheiaDevelopmentExecution`. The deployer
trusts GitHub OIDC for this repository's `theia-infrastructure` environment.
The trust condition uses GitHub's immutable owner and repository IDs. That
environment permits `main` and requires the owner's approval. The
execution role trusts CloudFormation and scopes writes to project-tagged host
resources, the reviewed AMI, Theia's runtime role and the named budget. It
cannot grant public ingress or attach a different policy to the runtime role.
The host CDK app uses the dedicated roles while retaining the existing Sydney
bootstrap. The shared bootstrap's roles and trust remain independent.

To review an IAM change separately from host resources, synthesize the
identity stack from this directory:

```sh
npm exec -- cdk synth --app 'node bin/theia-deployment-identity.ts' \
  --no-lookups --output cdk.out/identity --quiet
```

The identity stack has exactly two IAM resources. Its `ReviewedAmiId` must
match the host's approved `UbuntuAmiId`. Review IAM changes and role simulation
separately from the host's first live deployment. The protected environment
and role are ready for an approved deployment workflow. The host still needs
an actual CloudFormation deployment to qualify the complete provisioning path.

The identity stack was created on 24 November 2026 with termination
protection. The AWS policy simulator allowed the Theia change set and the
dedicated role pass, and denied the corresponding Projex change set, the
shared administrator role pass, changes to Projex-tagged resources, public
security-group ingress and attaching AdministratorAccess to the runtime role.
This validates the tested permissions separately from GitHub token exchange
and host provisioning.

The manual **Theia development host** workflow runs on `main` and uses the
protected `theia-infrastructure` environment. Configure its
`THEIA_AWS_ACCOUNT_ID` variable and `THEIA_BUDGET_ALERT_EMAIL` secret. Run it in
`verify` mode first. This mode checks the actual GitHub OIDC caller and asks
CloudFormation to validate the synthesized host template. It creates no AWS
resources. For the approved first deployment, dispatch `deploy` with the
reviewed AMI, zone and budget. After the same checks it submits the inline
CDK template to CloudFormation using the dedicated execution role. The
workflow fails if the stack already exists, so routine updates require a
separate review path.

## Review before provisioning

The stack requires explicit `UbuntuAmiId`, `HostAvailabilityZone`,
`MonthlyBudgetUsd` and `BudgetAlertEmail` parameters. Activate `Project` as an
AWS cost allocation tag when it becomes available and verify the filtered
budget receives the `TheiaMMO` charges. The preflight found no `Project` billing
tag record. Alerts at 50%, 80% and 100% are notifications. Until tagged charges
appear, check account billing directly.

The reviewed parameters are stored locally in `parameters.local.json`, which
Git ignores. The proposal uses one subnet in Sydney `ap-southeast-2a`
(`apse2-az1`) and a US$30 monthly budget. The thresholds are US$15, US$24 and
US$30. Changing the subnet's AZ later replaces the subnet and its host, so
keep the selected zone stable and verify a database restore before moving.

The 24 November 2026 Sydney on-demand rates were US$0.0475 per compute hour,
US$0.096 per gp3 GB-month and US$0.005 per IPv4 address-hour. At 730 hours per
month the disk and address cost about US$7.49 even with the instance stopped.
At 100 running hours the total is about US$12.24, or US$42.17 if left running
the full month. These estimates exclude traffic, taxes and later services.
The CDK stack should use the selected parameters from the approved review.

Use `npx cdk diff --method=template` for a template-only plan after confirming
the logged-in account. Deployment requires explicit approval. After the first
approved deployment, verify Systems Manager connectivity and encrypted
storage before installing any game runtime. Apply the local rehearsal's
`KillSignal=SIGINT` and bounded stop timeout to the game service, then verify
graceful flush via `systemctl stop` before exercising stop/start of the EC2
instance itself.

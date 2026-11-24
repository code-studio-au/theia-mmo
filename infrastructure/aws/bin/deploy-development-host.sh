#!/usr/bin/env bash
set -euo pipefail

mode=${1:?Choose verify or deploy}
if [[ "$mode" != verify && "$mode" != deploy ]]; then
  echo 'Choose verify or deploy.' >&2
  exit 1
fi
cd "$(dirname "${BASH_SOURCE[0]}")/.."
region=ap-southeast-2
stack=TheiaDevelopmentHost
account=${THEIA_AWS_ACCOUNT_ID:?Missing configured AWS account}
session=${THEIA_AWS_SESSION_NAME:?Missing expected GitHub role session name}
if [[ ! "$account" =~ ^[0-9]{12}$ || ! "$session" =~ ^theia-[0-9]+-[0-9]+$ ]]; then
  echo 'Invalid AWS account or deployment session name.' >&2
  exit 1
fi

identity=$(aws sts get-caller-identity --region "$region" --query Arn --output text)
if [[ "$identity" != "arn:aws:sts::$account:assumed-role/TheiaDevelopmentDeployer/$session" ]]; then
  echo 'GitHub did not assume the expected Theia deployment role.' >&2
  exit 1
fi
echo 'Verified GitHub OIDC as TheiaDevelopmentDeployer.'

template=cdk.out/TheiaDevelopmentHost.template.json
aws cloudformation validate-template --region "$region" --template-body "file://$template" >/dev/null
echo 'CloudFormation validated the synthesized Theia host template.'
if [[ "$mode" == verify ]]; then
  exit 0
fi

ami=${THEIA_UBUNTU_AMI_ID:?Missing approved Ubuntu AMI}
zone=${THEIA_HOST_AZ:?Missing approved availability zone}
budget=${THEIA_MONTHLY_BUDGET_USD:?Missing approved monthly budget}
recipient=${THEIA_BUDGET_ALERT_EMAIL:?Missing budget alert recipient}
if [[ ! "$ami" =~ ^ami-[0-9a-f]{17}$ || ! "$zone" =~ ^ap-southeast-2[a-z]$ || ! "$budget" =~ ^[1-9][0-9]*$ ]]; then
  echo 'Invalid deployment parameter.' >&2
  exit 1
fi

executor="arn:aws:iam::$account:role/theia/TheiaDevelopmentExecution"
umask 077
parameters=$(mktemp "${RUNNER_TEMP:-/tmp}/theia-host-parameters.XXXXXXXX")
trap 'rm -f -- "$parameters"' EXIT
jq -n --arg ami "$ami" --arg zone "$zone" --arg budget "$budget" --arg email "$recipient" '[
  {ParameterKey:"UbuntuAmiId", ParameterValue:$ami},
  {ParameterKey:"HostAvailabilityZone", ParameterValue:$zone},
  {ParameterKey:"MonthlyBudgetUsd", ParameterValue:$budget},
  {ParameterKey:"BudgetAlertEmail", ParameterValue:$email}
]' > "$parameters"

# This template has no application assets. Submit it directly to CloudFormation.
# CreateStack refuses to overwrite an existing host or accidentally update its AMI.
aws cloudformation create-stack --region "$region" --stack-name "$stack" \
  --template-body "file://$template" --parameters "file://$parameters" \
  --role-arn "$executor" --capabilities CAPABILITY_NAMED_IAM \
  --enable-termination-protection --on-failure ROLLBACK --timeout-in-minutes 12 \
  --tags Key=Project,Value=TheiaMMO Key=Environment,Value=development Key=ManagedBy,Value=CDK \
  >/dev/null
if ! aws cloudformation wait stack-create-complete --region "$region" --stack-name "$stack"; then
  aws cloudformation describe-stack-events --region "$region" --stack-name "$stack" \
    --max-items 12 --query 'StackEvents[].{resource:LogicalResourceId,status:ResourceStatus,reason:ResourceStatusReason}'
  exit 1
fi
aws cloudformation describe-stacks --region "$region" --stack-name "$stack" \
  --query 'Stacks[0].{stack:StackName,status:StackStatus,terminationProtection:EnableTerminationProtection}'

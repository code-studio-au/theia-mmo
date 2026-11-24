#!/usr/bin/env bash
set -euo pipefail

stack=TheiaDevelopmentRealmGateway
stack_arn_prefix="arn:aws:cloudformation:ap-southeast-2:${THEIA_AWS_ACCOUNT_ID:?}:stack/$stack/"
case "${1:-}" in
  prepare)
    readonly artifact=build/realm-gateway/bootstrap.zip
    readonly template=infrastructure/aws/cdk.out/TheiaDevelopmentRealmGateway.template.json
    (cd build/realm-gateway && sha256sum --check SHA256SUMS)
    digest=$(sha256sum "$artifact" | cut -d' ' -f1)
    object="theia/realm-gateway/$digest.zip"
    version=$(aws s3api put-object --bucket "${THEIA_GATEWAY_BUCKET:?}" --key "$object" \
      --body "$artifact" --checksum-algorithm SHA256 --query VersionId --output text)
    [[ -n "$version" && "$version" != None && "$version" != null ]] || {
      echo 'The artifact bucket must have versioning enabled' >&2
      exit 1
    }
    type=CREATE
    if aws cloudformation describe-stacks --stack-name "$stack" >/dev/null 2>&1; then
      type=UPDATE
    fi
    changes=$(aws cloudformation create-change-set --stack-name "$stack" \
      --change-set-name "theia-gateway-${GITHUB_RUN_ID:?}-${GITHUB_RUN_ATTEMPT:?}" \
      --change-set-type "$type" --role-arn "${THEIA_GATEWAY_EXECUTION_ROLE:?}" \
      --template-body "file://$template" --capabilities CAPABILITY_NAMED_IAM \
      --parameters "ParameterKey=InstanceId,ParameterValue=${THEIA_INSTANCE_ID:?}" \
        "ParameterKey=CognitoIssuer,ParameterValue=${THEIA_COGNITO_ISSUER:?}" \
        "ParameterKey=CognitoClientId,ParameterValue=${THEIA_COGNITO_CLIENT:?}" \
        "ParameterKey=ArtifactBucket,ParameterValue=$THEIA_GATEWAY_BUCKET" \
        "ParameterKey=ArtifactKey,ParameterValue=$object" \
        "ParameterKey=ArtifactVersion,ParameterValue=$version" --query Id --output text)
    aws cloudformation wait change-set-create-complete --stack-name "$stack" --change-set-name "$changes"
    aws cloudformation describe-change-set --stack-name "$stack" --change-set-name "$changes" \
      --query '{ChangeSet:ChangeSetId,Changes:Changes[].ResourceChange.{Action:Action,Resource:LogicalResourceId,Type:ResourceType,Replacement:Replacement}}'
    ;;
  execute)
    [[ "${THEIA_CHANGE_SET:?}" == arn:aws:cloudformation:ap-southeast-2:"$THEIA_AWS_ACCOUNT_ID":changeSet/theia-gateway-*/* ]]
    owner=$(aws cloudformation describe-change-set --stack-name "$stack" \
      --change-set-name "$THEIA_CHANGE_SET" --query StackId --output text)
    [[ "$owner" == "$stack_arn_prefix"* ]]
    before=$(aws cloudformation describe-stacks --stack-name "$stack" --query 'Stacks[0].StackStatus' --output text)
    aws cloudformation execute-change-set --stack-name "$stack" --change-set-name "$THEIA_CHANGE_SET"
    if [[ "$before" == REVIEW_IN_PROGRESS ]]; then
      aws cloudformation wait stack-create-complete --stack-name "$stack"
      aws cloudformation update-termination-protection --stack-name "$stack" --enable-termination-protection >/dev/null
    else
      aws cloudformation wait stack-update-complete --stack-name "$stack"
    fi
    aws cloudformation describe-stacks --stack-name "$stack" --query 'Stacks[0].Outputs'
    ;;
  *) echo 'Choose prepare or execute' >&2; exit 2 ;;
esac

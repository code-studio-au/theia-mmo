import assert from 'node:assert/strict';
import { test } from 'node:test';
import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { TheiaDeploymentIdentityStack } from '../lib/deployment-identity-stack.ts';
import { DEPLOY_ROLE, EXECUTION_ROLE, HOST_PROFILE, HOST_ROLE, REGION } from '../lib/names.ts';

const template = Template.fromStack(new TheiaDeploymentIdentityStack(
  new App({ analyticsReporting: false }), 'TestIdentity',
  { env: { account: '111111111111', region: REGION }, synthesizer: new BootstraplessSynthesizer() },
));

test('identity setup creates only two dedicated IAM roles', () => {
  assert.deepEqual(Object.values(template.toJSON().Resources).map((entry) =>
    (entry as { Type: string }).Type), ['AWS::IAM::Role', 'AWS::IAM::Role']);
  template.hasResourceProperties('AWS::IAM::Role', {
    RoleName: DEPLOY_ROLE,
    AssumeRolePolicyDocument: { Statement: [{
      Action: 'sts:AssumeRoleWithWebIdentity',
      Condition: { StringEquals: {
        'token.actions.githubusercontent.com:aud': 'sts.amazonaws.com',
        'token.actions.githubusercontent.com:sub': 'repo:code-studio-au@187219708/theia-mmo@1421547475:environment:theia-infrastructure',
      } },
    }] },
  });
  template.hasResourceProperties('AWS::IAM::Role', {
    RoleName: EXECUTION_ROLE,
    AssumeRolePolicyDocument: { Statement: [{
      Action: 'sts:AssumeRole',
      Principal: { Service: 'cloudformation.amazonaws.com' },
    }] },
  });
});

test('the deployer can prepare only the Theia stack with the dedicated executor', () => {
  const roles = template.findResources('AWS::IAM::Role');
  const deployer = Object.values(roles).find((entry) => entry.Properties.RoleName === DEPLOY_ROLE)!;
  const statements = deployer.Properties.Policies[0].PolicyDocument.Statement;
  const stackWrites = statements.find((statement: { Sid: string }) => statement.Sid === 'PrepareOnlyTheiaStack');
  assert.deepEqual(stackWrites.Resource, {
    'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' }, ':cloudformation:', { Ref: 'AWS::Region' },
      ':', { Ref: 'AWS::AccountId' }, ':stack/TheiaDevelopmentHost/*']],
  });
  assert.deepEqual(stackWrites.Condition, {
    ArnEquals: { 'cloudformation:RoleArn': { 'Fn::GetAtt': ['CloudFormationExecution', 'Arn'] } },
  });
  assert.ok(!statements.flatMap((statement: { Action: string[] }) => statement.Action).includes('cloudformation:DeleteStack'));
});

test('CloudFormation can check for the named runtime identities before creating them', () => {
  const roles = template.findResources('AWS::IAM::Role');
  const executor = Object.values(roles).find((entry) => entry.Properties.RoleName === EXECUTION_ROLE)!;
  const statements = executor.Properties.Policies[0].PolicyDocument.Statement;
  for (const [sid, action, resource] of [
    ['CheckRuntimeRoleBeforeCreation', 'iam:GetRole', `role/${HOST_ROLE}`],
    ['CheckRuntimeProfileBeforeCreation', 'iam:GetInstanceProfile', `instance-profile/${HOST_PROFILE}`],
  ]) {
    assert.deepEqual(statements.find((statement: { Sid: string }) => statement.Sid === sid), {
      Sid: sid,
      Effect: 'Allow',
      Action: [action],
      Resource: {
        'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' }, ':iam::',
          { Ref: 'AWS::AccountId' }, `:${resource}`]],
      },
    });
  }
});

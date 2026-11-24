import assert from 'node:assert/strict';
import { test } from 'node:test';
import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { TheiaDeploymentIdentityStack } from '../lib/deployment-identity-stack.ts';
import { DEPLOY_ROLE, EXECUTION_ROLE, GITHUB_ENVIRONMENT, GITHUB_REPOSITORY, REGION } from '../lib/names.ts';

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
        'token.actions.githubusercontent.com:sub': `repo:${GITHUB_REPOSITORY}:environment:${GITHUB_ENVIRONMENT}`,
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

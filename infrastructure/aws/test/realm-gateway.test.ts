import { App } from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import assert from 'node:assert/strict';
import test from 'node:test';
import { RealmGatewayIdentityStack } from '../lib/realm-gateway-identity-stack.ts';
import { RealmGatewayStack } from '../lib/realm-gateway-stack.ts';
import { GITHUB_ENVIRONMENT, GITHUB_SUBJECT_REPOSITORY, gatewayExport } from '../lib/names.ts';

const env = { account: '111111111111', region: 'ap-southeast-2' };
const imported = (name: string) => ({ 'Fn::ImportValue': gatewayExport(name) });

test('all gateway routes are scoped to the imported API and require JWTs', () => {
  const template = Template.fromStack(new RealmGatewayStack(new App(), 'TestGateway', { env }));
  template.resourceCountIs('AWS::ApiGatewayV2::Api', 0);
  template.resourceCountIs('AWS::IAM::Role', 0);
  template.resourceCountIs('AWS::IAM::Policy', 0);
  template.resourceCountIs('AWS::ApiGatewayV2::Route', 4);
  for (const route of ['GET /v1/realm/status', 'POST /v1/realm/wake',
    'POST /v1/realm/reservation', 'DELETE /v1/realm/reservation']) {
    template.hasResourceProperties('AWS::ApiGatewayV2::Route', {
      ApiId: imported('ApiId'), RouteKey: route, AuthorizationType: 'JWT',
      AuthorizationScopes: ['theia-development/play'],
    });
  }
  template.hasResourceProperties('AWS::DynamoDB::Table', { BillingMode: 'PAY_PER_REQUEST' });
  template.hasResourceProperties('AWS::Lambda::Function', {
    Role: imported('LambdaRuntimeArn'),
    Environment: { Variables: { THEIA_INSTANCE_ID: imported('InstanceId') } },
    Code: { S3Bucket: imported('ArtifactBucket'), S3ObjectVersion: { Ref: 'ArtifactVersion' } },
  });
});

test('identity owns an unpublished API and separates deployment, execution and runtime privileges', () => {
  const template = Template.fromStack(new RealmGatewayIdentityStack(new App(), 'TestIdentity', { env }));
  template.resourceCountIs('AWS::ApiGatewayV2::Api', 1);
  template.resourceCountIs('AWS::ApiGatewayV2::Stage', 0);
  template.resourceCountIs('AWS::ApiGatewayV2::Route', 0);
  template.resourceCountIs('AWS::IAM::Role', 3);
  template.hasResourceProperties('AWS::IAM::Role', {
    RoleName: 'TheiaRealmGatewayDeployer',
    AssumeRolePolicyDocument: { Statement: Match.arrayWith([Match.objectLike({
      Condition: { StringEquals: {
        'token.actions.githubusercontent.com:aud': 'sts.amazonaws.com',
        'token.actions.githubusercontent.com:sub': `repo:${GITHUB_SUBJECT_REPOSITORY}:environment:${GITHUB_ENVIRONMENT}`,
      } },
    })]) },
  });
  const roles = Object.values(template.findResources('AWS::IAM::Role')).map(r => r.Properties);
  const [runtime, execution, deploy] = ['TheiaRealmGateway', 'TheiaRealmGatewayExecution', 'TheiaRealmGatewayDeployer']
    .map(name => roles.find(role => role.RoleName === name));
  const statements = (role: typeof runtime) => role.Policies.flatMap(
    (policy: { PolicyDocument: { Statement: Record<string, string | string[]>[] } }) => policy.PolicyDocument.Statement);
  const allows = (role: typeof runtime, action: string) => statements(role).find(
    (statement: Record<string, string | string[]>) => [statement.Action].flat().includes(action));
  assert.deepEqual(allows(runtime, 'ec2:StartInstances')?.Resource,
    { 'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' }, ':ec2:', { Ref: 'AWS::Region' }, ':', { Ref: 'AWS::AccountId' }, ':instance/', { Ref: 'InstanceId' }]] });
  assert.ok(allows(runtime, 'dynamodb:PutItem'));
  assert.ok(!allows(runtime, 'ec2:StopInstances') && !allows(runtime, 'iam:PassRole'));
  assert.ok(!allows(execution, 'iam:PutRolePolicy') && !allows(execution, 'iam:CreateRole'));
  assert.ok(!allows(execution, 'dynamodb:DeleteTable') && !allows(execution, 'ec2:StartInstances'));
  assert.ok(!allows(deploy, 'lambda:CreateFunction') && !allows(deploy, 'ec2:StartInstances'));
  assert.match(JSON.stringify(allows(execution, 'apigateway:POST')?.Resource), /RealmApi/);
  assert.match(JSON.stringify(allows(deploy, 'cloudformation:CreateChangeSet')?.Resource), /stack\/TheiaDevelopmentRealmGateway\/\*/);
  assert.match(JSON.stringify(allows(deploy, 's3:PutObject')?.Resource), /theia\/realm-gateway\/\*\.zip/);
});

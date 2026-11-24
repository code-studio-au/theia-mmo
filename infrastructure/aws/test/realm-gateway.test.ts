import { App } from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import assert from 'node:assert/strict';
import test from 'node:test';
import { RealmGatewayStack } from '../lib/realm-gateway-stack.ts';

test('gateway routes require scoped JWTs and only this instance can be started', () => {
  const stack = new RealmGatewayStack(new App(), 'TestGateway', {
    env: { account: '111111111111', region: 'ap-southeast-2' },
  });
  const template = Template.fromStack(stack);
  template.resourceCountIs('AWS::ApiGatewayV2::Route', 4);
  for (const route of ['GET /v1/realm/status', 'POST /v1/realm/wake',
    'POST /v1/realm/reservation', 'DELETE /v1/realm/reservation']) {
    template.hasResourceProperties('AWS::ApiGatewayV2::Route', {
      RouteKey: route, AuthorizationType: 'JWT', AuthorizationScopes: ['theia-development/play'],
    });
  }
  template.hasResourceProperties('AWS::DynamoDB::Table', { BillingMode: 'PAY_PER_REQUEST' });
  template.hasResourceProperties('AWS::IAM::Policy', {
    PolicyDocument: { Statement: Match.arrayWith([{
      Action: 'ec2:StartInstances', Effect: 'Allow',
      Resource: { 'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' }, ':ec2:ap-southeast-2:111111111111:instance/', { Ref: 'InstanceId' }]] },
    }]) },
  });
  assert.ok(!JSON.stringify(template.toJSON()).includes('ec2:StopInstances'));
});

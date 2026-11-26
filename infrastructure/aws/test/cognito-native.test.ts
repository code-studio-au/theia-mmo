import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import test from 'node:test';
import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { TheiaCognitoStack } from '../lib/cognito-stack.ts';
import { TheiaCognitoNativeStack } from '../lib/cognito-native-stack.ts';

const env = { account: '111111111111', region: 'ap-southeast-2' };

test('native SRP and access token customization are opt-in on the existing public Cognito client', () => {
  const template = Template.fromStack(new TheiaCognitoStack(new App(), 'Cognito', { env }));
  template.hasParameter('NativePlayScopeArn', { Default: '' });
  template.hasResourceProperties('AWS::Cognito::UserPool', {
    UserPoolTier: 'ESSENTIALS',
    AdminCreateUserConfig: { AllowAdminCreateUserOnly: true },
    AccountRecoverySetting: { RecoveryMechanisms: [{ Name: 'verified_email', Priority: 1 }] },
    MfaConfiguration: 'OPTIONAL', EnabledMfas: ['SOFTWARE_TOKEN_MFA'],
    LambdaConfig: { 'Fn::If': ['EnableNativeSignIn', {
      PreTokenGenerationConfig: { LambdaArn: { Ref: 'NativePlayScopeArn' }, LambdaVersion: 'V2_0' },
    }, { Ref: 'AWS::NoValue' }] },
  });
  template.hasResourceProperties('AWS::Cognito::UserPoolClient', {
    GenerateSecret: false,
    AllowedOAuthFlows: ['code'],
    AllowedOAuthScopes: ['openid', 'theia-development/play'],
    ExplicitAuthFlows: { 'Fn::If': ['EnableNativeSignIn',
      ['ALLOW_REFRESH_TOKEN_AUTH', 'ALLOW_USER_SRP_AUTH'], ['ALLOW_REFRESH_TOKEN_AUTH']] },
  });
});

test('the website account client can authenticate with SRP and carries no play scope', () => {
  const template = Template.fromStack(new TheiaCognitoStack(new App(), 'Cognito', { env }));
  const client = template.findResources('AWS::Cognito::UserPoolClient', {
    Properties: { ClientName: 'Theia Online account setup' },
  }).WebsiteAccount.Properties;
  assert.equal(client.GenerateSecret, false);
  assert.equal(client.AllowedOAuthFlowsUserPoolClient, false);
  assert.equal(client.AllowedOAuthScopes, undefined);
  assert.deepEqual(client.ExplicitAuthFlows, ['ALLOW_REFRESH_TOKEN_AUTH', 'ALLOW_USER_SRP_AUTH']);
});

test('only the designated Cognito pool can invoke the native play scope trigger', () => {
  const template = Template.fromStack(new TheiaCognitoNativeStack(new App(), 'Native', { env }));
  template.hasResourceProperties('AWS::Lambda::Function', {
    Environment: { Variables: { THEIA_COGNITO_CLIENT: { Ref: 'ExistingLauncherClientId' } } },
  });
  template.resourceCountIs('AWS::Lambda::Permission', 1);
  template.hasResourceProperties('AWS::Lambda::Permission', {
    Action: 'lambda:InvokeFunction', Principal: 'cognito-idp.amazonaws.com', SourceAccount: { Ref: 'AWS::AccountId' },
    SourceArn: { 'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' },
      ':cognito-idp:ap-southeast-2:111111111111:userpool/', { Ref: 'ExistingPoolId' }]] },
  });
});

test('the play scope is restricted to tester tokens for the public launcher client', async () => {
  const { handler } = createRequire(import.meta.url)('../lib/cognito-play-scope.cjs');
  const event = (triggerSource: string, clientId = 'launcher', groups = ['theia-testers'], scopes: string[] = []) => ({
    triggerSource, callerContext: { clientId },
    request: { groupConfiguration: { groupsToOverride: groups }, scopes }, response: {},
  });
  const original = process.env.THEIA_COGNITO_CLIENT;
  process.env.THEIA_COGNITO_CLIENT = 'launcher';
  try {
    for (const source of ['TokenGeneration_Authentication', 'TokenGeneration_NewPasswordChallenge',
      'TokenGeneration_RefreshTokens']) {
      const result = await handler(event(source));
      assert.deepEqual(result.response, { claimsAndScopeOverrideDetails: {
        accessTokenGeneration: { scopesToAdd: ['theia-development/play'] },
      } });
    }
    for (const request of [event('TokenGeneration_HostedAuth'),
      event('TokenGeneration_Authentication', 'another-client'),
      event('TokenGeneration_Authentication', 'launcher', ['another-group']),
      event('TokenGeneration_RefreshTokens', 'launcher', ['theia-testers'], ['theia-development/play'])]) {
      assert.deepEqual((await handler(request)).response, {});
    }
  } finally {
    if (original === undefined) delete process.env.THEIA_COGNITO_CLIENT;
    else process.env.THEIA_COGNITO_CLIENT = original;
  }
});

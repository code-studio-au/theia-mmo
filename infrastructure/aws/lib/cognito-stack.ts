import { Aws, CfnCondition, CfnOutput, CfnParameter, Fn, RemovalPolicy, Stack, Tags, aws_cognito as cognito } from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { PROJECT } from './names.ts';

export const COGNITO_STACK = 'TheiaDevelopmentCognito';
export const CALLBACK = 'http://127.0.0.1:38691/callback';
export const TESTER_GROUP = 'theia-testers';
const RESOURCE = 'theia-development';
const SCOPE = 'play';
const ACCOUNT_URL = 'https://theiaonline.com/account/';

export class TheiaCognitoStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');
    Tags.of(this).add('ManagedBy', 'CDK');

    const prefix = new CfnParameter(this, 'DomainPrefix', {
      type: 'String', allowedPattern: '^[a-z0-9][a-z0-9-]{2,61}[a-z0-9]$',
    });
    const scopeArn = new CfnParameter(this, 'NativePlayScopeArn', {
      type: 'String', default: '',
      allowedPattern: '(arn:aws:lambda:ap-southeast-2:[0-9]{12}:function:theia-development-native-play-scope)?',
      description: 'Set only after the native sign-in stack grants this existing pool access.',
    }).valueAsString;
    const native = new CfnCondition(this, 'EnableNativeSignIn', {
      expression: Fn.conditionNot(Fn.conditionEquals(scopeArn, '')),
    });
    const pool = new cognito.CfnUserPool(this, 'UserPool', {
      userPoolName: 'Theia development testers',
      userPoolTier: 'ESSENTIALS',
      lambdaConfig: Fn.conditionIf(native.logicalId, {
        PreTokenGenerationConfig: { LambdaArn: scopeArn, LambdaVersion: 'V2_0' },
      }, Aws.NO_VALUE) as unknown as cognito.CfnUserPool.LambdaConfigProperty,
      deletionProtection: 'ACTIVE',
      usernameConfiguration: { caseSensitive: false },
      adminCreateUserConfig: {
        allowAdminCreateUserOnly: true,
        inviteMessageTemplate: {
          emailSubject: 'Your Theia Online playtest invitation',
          emailMessage: `Set up your account at ${ACCOUNT_URL}. Your username is {username} and your temporary password is {####}.`,
        },
      },
      autoVerifiedAttributes: ['email'],
      userAttributeUpdateSettings: { attributesRequireVerificationBeforeUpdate: ['email'] },
      accountRecoverySetting: { recoveryMechanisms: [{ name: 'verified_email', priority: 1 }] },
      mfaConfiguration: 'OPTIONAL',
      enabledMfas: ['SOFTWARE_TOKEN_MFA'],
      policies: {
        passwordPolicy: {
          minimumLength: 16, requireLowercase: true, requireUppercase: true,
          requireNumbers: true, requireSymbols: true, temporaryPasswordValidityDays: 7,
        },
        signInPolicy: { allowedFirstAuthFactors: ['PASSWORD'] },
      },
    });
    pool.applyRemovalPolicy(RemovalPolicy.RETAIN);
    const resource = new cognito.CfnUserPoolResourceServer(this, 'PlayScope', {
      userPoolId: pool.ref, identifier: RESOURCE, name: 'Theia development realm',
      scopes: [{ scopeName: SCOPE, scopeDescription: 'Join the Theia development realm' }],
    });
    const client = new cognito.CfnUserPoolClient(this, 'Launcher', {
      userPoolId: pool.ref, clientName: 'Theia Linux launcher', generateSecret: false,
      supportedIdentityProviders: ['COGNITO'],
      allowedOAuthFlowsUserPoolClient: true, allowedOAuthFlows: ['code'],
      allowedOAuthScopes: ['openid', `${RESOURCE}/${SCOPE}`], callbackUrLs: [CALLBACK],
      explicitAuthFlows: Fn.conditionIf(native.logicalId,
        ['ALLOW_REFRESH_TOKEN_AUTH', 'ALLOW_USER_SRP_AUTH'], ['ALLOW_REFRESH_TOKEN_AUTH'],
      ) as unknown as string[],
      preventUserExistenceErrors: 'ENABLED', enableTokenRevocation: true,
      accessTokenValidity: 1, idTokenValidity: 1, refreshTokenValidity: 1,
      tokenValidityUnits: { accessToken: 'hours', idToken: 'hours', refreshToken: 'days' },
    });
    client.addResourceDependency(resource);
    const websiteClient = new cognito.CfnUserPoolClient(this, 'WebsiteAccount', {
      userPoolId: pool.ref, clientName: 'Theia Online account setup', generateSecret: false,
      allowedOAuthFlowsUserPoolClient: false,
      explicitAuthFlows: ['ALLOW_REFRESH_TOKEN_AUTH', 'ALLOW_USER_SRP_AUTH'],
      readAttributes: ['email', 'email_verified'], writeAttributes: ['email'],
      preventUserExistenceErrors: 'ENABLED', enableTokenRevocation: true,
      accessTokenValidity: 15, idTokenValidity: 15, refreshTokenValidity: 1,
      tokenValidityUnits: { accessToken: 'minutes', idToken: 'minutes', refreshToken: 'days' },
    });
    const domain = new cognito.CfnUserPoolDomain(this, 'ManagedDomain', {
      userPoolId: pool.ref, domain: prefix.valueAsString, managedLoginVersion: 2,
    });
    const branding = new cognito.CfnManagedLoginBranding(this, 'LauncherBranding', {
      userPoolId: pool.ref, clientId: client.ref, useCognitoProvidedValues: true,
    });
    branding.addResourceDependency(domain);
    new cognito.CfnUserPoolGroup(this, 'Testers', {
      userPoolId: pool.ref, groupName: TESTER_GROUP,
    });

    new CfnOutput(this, 'PoolId', { value: pool.ref });
    new CfnOutput(this, 'LauncherClientId', { value: client.ref });
    new CfnOutput(this, 'WebsiteAccountClientId', { value: websiteClient.ref });
    new CfnOutput(this, 'Issuer', { value: `https://cognito-idp.${Aws.REGION}.${Aws.URL_SUFFIX}/${pool.ref}` });
    new CfnOutput(this, 'ManagedLogin', { value: `https://${domain.ref}.auth.${Aws.REGION}.amazoncognito.com` });
    new CfnOutput(this, 'RedirectUri', { value: CALLBACK });
  }
}

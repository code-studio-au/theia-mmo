import { readFileSync } from 'node:fs';
import {
  ArnFormat, Aws, CfnOutput, CfnParameter, Duration, Stack, Tags,
  aws_lambda as lambda, aws_iam as iam, aws_logs as logs,
} from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { PROJECT } from './names.ts';

export const NATIVE_COGNITO_STACK = 'TheiaDevelopmentNativeSignIn';

export class TheiaCognitoNativeStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');

    const pool = new CfnParameter(this, 'ExistingPoolId', {
      type: 'String', allowedPattern: 'ap-southeast-2_[A-Za-z0-9]+',
    }).valueAsString;
    const client = new CfnParameter(this, 'ExistingLauncherClientId', {
      type: 'String', allowedPattern: '[A-Za-z0-9]{1,128}',
    }).valueAsString;
    const group = new logs.LogGroup(this, 'ScopeLogs', {
      logGroupName: '/aws/lambda/theia-development-native-play-scope', retention: logs.RetentionDays.ONE_WEEK,
    });
    const role = new iam.Role(this, 'ScopeRole', { assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com') });
    group.grantWrite(role);
    const fn = new lambda.Function(this, 'PlayScope', {
      functionName: 'theia-development-native-play-scope',
      runtime: lambda.Runtime.NODEJS_22_X,
      handler: 'index.handler',
      code: lambda.Code.fromInline(readFileSync(new URL('./cognito-play-scope.cjs', import.meta.url), 'utf8')),
      environment: { THEIA_COGNITO_CLIENT: client },
      role, logGroup: group, timeout: Duration.seconds(3), memorySize: 128,
    });
    fn.addPermission('ExistingSydneyPool', {
      principal: new iam.ServicePrincipal('cognito-idp.amazonaws.com'),
      sourceAccount: Aws.ACCOUNT_ID,
      sourceArn: this.formatArn({
        service: 'cognito-idp', resource: 'userpool', resourceName: pool,
        arnFormat: ArnFormat.SLASH_RESOURCE_NAME,
      }),
    });
    new CfnOutput(this, 'NativePlayScopeArn', { value: fn.functionArn });
  }
}

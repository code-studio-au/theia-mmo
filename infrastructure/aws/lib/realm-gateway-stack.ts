import {
  Aws, CfnOutput, CfnParameter, Duration, Fn, RemovalPolicy, Stack, Tags,
  aws_apigatewayv2 as api, aws_dynamodb as dynamodb, aws_iam as iam, aws_lambda as lambda, aws_s3 as s3,
} from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { GATEWAY_ARTIFACT_PREFIX, GATEWAY_FUNCTION, GATEWAY_TABLE, PROJECT, gatewayExport } from './names.ts';

export class RealmGatewayStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');

    const parameter = (name: string, allowedPattern: string) =>
      new CfnParameter(this, name, { type: 'String', allowedPattern }).valueAsString;
    const instance = Fn.importValue(gatewayExport('InstanceId'));
    const issuer = parameter('CognitoIssuer', 'https://cognito-idp\\.ap-southeast-2\\.amazonaws\\.com/ap-southeast-2_[A-Za-z0-9]+');
    const client = parameter('CognitoClientId', '[A-Za-z0-9]{1,128}');
    const bucket = Fn.importValue(gatewayExport('ArtifactBucket'));
    const version = new CfnParameter(this, 'ArtifactVersion', { type: 'String', minLength: 1 }).valueAsString;
    const key = parameter('ArtifactKey', `${GATEWAY_ARTIFACT_PREFIX}[a-f0-9]{64}\\.zip`);

    const table = new dynamodb.Table(this, 'State', {
      tableName: GATEWAY_TABLE,
      partitionKey: { name: 'realm', type: dynamodb.AttributeType.STRING },
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
      removalPolicy: RemovalPolicy.RETAIN,
    });
    const role = iam.Role.fromRoleArn(this, 'GatewayRole',
      Fn.importValue(gatewayExport('LambdaRuntimeArn')), { mutable: false });
    const handler = new lambda.Function(this, 'Gateway', {
      functionName: GATEWAY_FUNCTION,
      runtime: lambda.Runtime.PROVIDED_AL2023,
      architecture: lambda.Architecture.X86_64,
      handler: 'bootstrap',
      role,
      code: lambda.Code.fromBucket(s3.Bucket.fromBucketName(this, 'Artifacts', bucket), key, version),
      memorySize: 256,
      timeout: Duration.seconds(20),
      reservedConcurrentExecutions: 4,
      environment: {
        THEIA_REALM_TABLE: table.tableName, THEIA_INSTANCE_ID: instance,
        THEIA_COGNITO_ISSUER: issuer, THEIA_COGNITO_CLIENT: client,
      },
    });
    const apiId = Fn.importValue(gatewayExport('ApiId'));
    const authorizer = new api.CfnAuthorizer(this, 'Tester', {
      apiId, name: 'Tester', authorizerType: 'JWT',
      identitySource: ['$request.header.Authorization'],
      jwtConfiguration: { audience: [client], issuer },
    });
    const integration = new api.CfnIntegration(this, 'GatewayIntegration', {
      apiId, integrationType: 'AWS_PROXY', integrationUri: handler.functionArn,
      payloadFormatVersion: '2.0',
    });
    new lambda.CfnPermission(this, 'InvokeFromRealmApi', {
      action: 'lambda:InvokeFunction', functionName: handler.functionArn,
      principal: 'apigateway.amazonaws.com',
      sourceArn: `arn:${Aws.PARTITION}:execute-api:${Aws.REGION}:${Aws.ACCOUNT_ID}:${apiId}/*/*/v1/realm/*`,
      sourceAccount: Aws.ACCOUNT_ID,
    });
    for (const [id, route] of [
      ['Status', 'GET /v1/realm/status'],
      ['Wake', 'POST /v1/realm/wake'],
      ['Renew', 'POST /v1/realm/reservation'],
      ['Release', 'DELETE /v1/realm/reservation'],
    ]) {
      new api.CfnRoute(this, `Route${id}`, {
        apiId, routeKey: route, authorizationType: 'JWT', authorizerId: authorizer.ref,
        authorizationScopes: ['theia-development/play'], target: `integrations/${integration.ref}`,
      });
    }
    new api.CfnStage(this, 'DefaultStage', {
      apiId,
      stageName: '$default',
      autoDeploy: true,
      defaultRouteSettings: { throttlingRateLimit: 4, throttlingBurstLimit: 8 },
    });
    new CfnOutput(this, 'GatewayOrigin', {
      value: `https://${apiId}.execute-api.${Aws.REGION}.${Aws.URL_SUFFIX}`,
    });
    new CfnOutput(this, 'RealmTable', { value: table.tableName });
  }
}

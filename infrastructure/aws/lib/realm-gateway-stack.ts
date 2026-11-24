import {
  ArnFormat, CfnOutput, CfnParameter, Duration, RemovalPolicy, Stack, Tags,
  aws_apigatewayv2 as api, aws_dynamodb as dynamodb, aws_iam as iam, aws_lambda as lambda, aws_s3 as s3,
} from 'aws-cdk-lib';
import { HttpJwtAuthorizer } from 'aws-cdk-lib/aws-apigatewayv2-authorizers';
import { HttpLambdaIntegration } from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { HOST_ROLE, PROJECT, ROLE_PATH } from './names.ts';

export class RealmGatewayStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');

    const parameter = (name: string, allowedPattern: string) =>
      new CfnParameter(this, name, { type: 'String', allowedPattern }).valueAsString;
    const instance = parameter('InstanceId', 'i-([0-9a-f]{8}|[0-9a-f]{17})');
    const issuer = parameter('CognitoIssuer', 'https://cognito-idp\\.ap-southeast-2\\.amazonaws\\.com/ap-southeast-2_[A-Za-z0-9]+');
    const client = parameter('CognitoClientId', '[A-Za-z0-9]{1,128}');
    const bucket = parameter('ArtifactBucket', '[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]');
    const version = new CfnParameter(this, 'ArtifactVersion', { type: 'String', minLength: 1 }).valueAsString;
    const key = parameter('ArtifactKey', 'theia/realm-gateway/[a-f0-9]{64}\\.zip');

    const table = new dynamodb.Table(this, 'State', {
      tableName: 'TheiaDevelopmentRealm',
      partitionKey: { name: 'realm', type: dynamodb.AttributeType.STRING },
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
      removalPolicy: RemovalPolicy.RETAIN,
    });
    const realmAccess = () => new iam.PolicyStatement({
      actions: ['dynamodb:GetItem', 'dynamodb:PutItem'],
      resources: [table.tableArn],
      conditions: { 'ForAllValues:StringEquals': { 'dynamodb:LeadingKeys': ['sydney-development'] } },
    });
    const role = new iam.Role(this, 'GatewayRole', {
      roleName: 'TheiaRealmGateway',
      path: ROLE_PATH,
      assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
      managedPolicies: [iam.ManagedPolicy.fromAwsManagedPolicyName('service-role/AWSLambdaBasicExecutionRole')],
    });
    const instanceArn = this.formatArn({
      service: 'ec2', resource: 'instance', resourceName: instance, arnFormat: ArnFormat.SLASH_RESOURCE_NAME,
    });
    role.addToPolicy(realmAccess());
    role.addToPolicy(new iam.PolicyStatement({ actions: ['ec2:DescribeInstances'], resources: ['*'] }));
    role.addToPolicy(new iam.PolicyStatement({ actions: ['ec2:StartInstances'], resources: [instanceArn] }));
    new iam.CfnPolicy(this, 'HostStateAccess', {
      policyName: 'TheiaDevelopmentRealmState',
      roles: [HOST_ROLE],
      policyDocument: new iam.PolicyDocument({ statements: [realmAccess()] }).toJSON(),
    });
    const handler = new lambda.Function(this, 'Gateway', {
      functionName: 'theia-development-realm-gateway',
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
    const authorizer = new HttpJwtAuthorizer('Tester', issuer, { jwtAudience: [client] });
    const http = new api.HttpApi(this, 'Api', { apiName: 'theia-development-realm', createDefaultStage: false });
    for (const [route, methods] of [
      ['status', [api.HttpMethod.GET]],
      ['wake', [api.HttpMethod.POST]],
      ['reservation', [api.HttpMethod.POST, api.HttpMethod.DELETE]],
    ] as const) {
      http.addRoutes({
        path: `/v1/realm/${route}`, methods: [...methods], authorizer,
        authorizationScopes: ['theia-development/play'],
        integration: new HttpLambdaIntegration(`Realm-${route}`, handler),
      });
    }
    new api.CfnStage(this, 'DefaultStage', {
      apiId: http.apiId,
      stageName: '$default',
      autoDeploy: true,
      defaultRouteSettings: { throttlingRateLimit: 4, throttlingBurstLimit: 8 },
    });
    new CfnOutput(this, 'GatewayOrigin', { value: http.apiEndpoint });
    new CfnOutput(this, 'RealmTable', { value: table.tableName });
  }
}

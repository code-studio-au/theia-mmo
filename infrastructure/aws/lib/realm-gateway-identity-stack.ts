import {
  Aws, CfnOutput, CfnParameter, Stack, Tags,
  aws_apigatewayv2 as api, aws_iam as iam,
} from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import {
  GATEWAY_ARTIFACT_PREFIX, GATEWAY_DEPLOY_ROLE, GATEWAY_EXECUTION_ROLE, GATEWAY_FUNCTION,
  GATEWAY_RUNTIME_ROLE, GATEWAY_STACK, GATEWAY_TABLE, GITHUB_ENVIRONMENT, GITHUB_SUBJECT_REPOSITORY,
  HOST_ROLE, PROJECT, ROLE_PATH, gatewayExport,
} from './names.ts';

export class RealmGatewayIdentityStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');

    const instance = new CfnParameter(this, 'InstanceId', {
      type: 'String', allowedPattern: 'i-([a-f0-9]{8}|[a-f0-9]{17})',
    }).valueAsString;
    const bucket = new CfnParameter(this, 'ArtifactBucket', {
      type: 'String', allowedPattern: '[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]',
    }).valueAsString;
    const arn = (service: string, resource: string) =>
      `arn:${Aws.PARTITION}:${service}:${Aws.REGION}:${Aws.ACCOUNT_ID}:${resource}`;
    const iamArn = (resource: string) => `arn:${Aws.PARTITION}:iam::${Aws.ACCOUNT_ID}:${resource}`;
    const table = arn('dynamodb', `table/${GATEWAY_TABLE}`);
    const fn = arn('lambda', `function:${GATEWAY_FUNCTION}`);
    const artifact = `arn:${Aws.PARTITION}:s3:::${bucket}/${GATEWAY_ARTIFACT_PREFIX}*.zip`;
    const logGroup = arn('logs', `log-group:/aws/lambda/${GATEWAY_FUNCTION}`);
    const realmAccess = () => new iam.PolicyStatement({
      actions: ['dynamodb:GetItem', 'dynamodb:PutItem'],
      resources: [table],
      conditions: {
        Null: { 'dynamodb:LeadingKeys': 'false' },
        'ForAllValues:StringEquals': { 'dynamodb:LeadingKeys': ['sydney-development'] },
      },
    });

    // The API gets its generated ID here so the workflow can manage routes on
    // precisely this API. It has no deployed stage until the gateway rollout.
    const http = new api.CfnApi(this, 'RealmApi', {
      name: 'theia-development-realm',
      protocolType: 'HTTP',
    });
    const apiArn = `arn:${Aws.PARTITION}:apigateway:${Aws.REGION}::/apis/${http.ref}`;
    const runtime = new iam.Role(this, 'LambdaRuntime', {
      roleName: GATEWAY_RUNTIME_ROLE,
      path: ROLE_PATH,
      assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
      inlinePolicies: {
        RealmRuntime: new iam.PolicyDocument({ statements: [
          realmAccess(),
          new iam.PolicyStatement({
            actions: ['ec2:DescribeInstances'], resources: ['*'],
            conditions: { StringEquals: { 'aws:RequestedRegion': Aws.REGION } },
          }),
          new iam.PolicyStatement({
            actions: ['ec2:StartInstances'], resources: [arn('ec2', `instance/${instance}`)],
          }),
          new iam.PolicyStatement({ actions: ['logs:CreateLogGroup'], resources: [logGroup] }),
          new iam.PolicyStatement({
            actions: ['logs:CreateLogStream', 'logs:PutLogEvents'],
            resources: [`${logGroup}:log-stream:*`],
          }),
        ] }),
      },
    });
    new iam.CfnPolicy(this, 'HostStateAccess', {
      policyName: 'TheiaDevelopmentRealmState', roles: [HOST_ROLE],
      policyDocument: new iam.PolicyDocument({ statements: [realmAccess()] }).toJSON(),
    });

    // Deliberately grant no IAM policy mutation to CloudFormation. IAM changes
    // remain on this separately reviewed administrator-owned stack.
    const execution = new iam.Role(this, 'CloudFormationExecution', {
      roleName: GATEWAY_EXECUTION_ROLE, path: ROLE_PATH,
      assumedBy: new iam.ServicePrincipal('cloudformation.amazonaws.com'),
      inlinePolicies: {
        GatewayResources: new iam.PolicyDocument({ statements: [
          new iam.PolicyStatement({
            actions: ['apigateway:GET'], resources: [apiArn, `${apiArn}/*`],
          }),
          new iam.PolicyStatement({
            actions: ['apigateway:POST', 'apigateway:PATCH', 'apigateway:DELETE'],
            resources: ['authorizers', 'integrations', 'routes', 'stages'].flatMap(kind =>
              [`${apiArn}/${kind}`, `${apiArn}/${kind}/*`]),
          }),
          new iam.PolicyStatement({
            // CreateStage also authorizes the tags against the stage collection.
            actions: ['apigateway:TagResource', 'apigateway:UntagResource'],
            resources: [`${apiArn}/stages`, `${apiArn}/stages/$default`],
          }),
          new iam.PolicyStatement({
            actions: [
              'lambda:CreateFunction', 'lambda:GetFunction', 'lambda:GetFunctionConfiguration',
              'lambda:UpdateFunctionCode', 'lambda:UpdateFunctionConfiguration', 'lambda:DeleteFunction',
              'lambda:GetFunctionConcurrency', 'lambda:PutFunctionConcurrency', 'lambda:DeleteFunctionConcurrency',
              'lambda:GetPolicy', 'lambda:RemovePermission', 'lambda:ListTags', 'lambda:TagResource', 'lambda:UntagResource',
            ],
            resources: [fn],
          }),
          new iam.PolicyStatement({
            actions: ['lambda:AddPermission'], resources: [fn],
            conditions: { StringEquals: { 'lambda:Principal': 'apigateway.amazonaws.com' } },
          }),
          new iam.PolicyStatement({
            actions: ['iam:PassRole'], resources: [runtime.roleArn],
            conditions: { StringEquals: { 'iam:PassedToService': 'lambda.amazonaws.com' } },
          }),
          new iam.PolicyStatement({
            actions: [
              'dynamodb:CreateTable', 'dynamodb:DescribeTable', 'dynamodb:UpdateTable',
              'dynamodb:ListTagsOfResource', 'dynamodb:TagResource', 'dynamodb:UntagResource',
              'dynamodb:DescribeContinuousBackups', 'dynamodb:DescribeTimeToLive',
            ],
            resources: [table],
          }),
          new iam.PolicyStatement({ actions: ['s3:GetObjectVersion'], resources: [artifact] }),
        ] }),
      },
    });

    const oidc = 'token.actions.githubusercontent.com';
    const deploy = new iam.Role(this, 'GitHubDeployer', {
      roleName: GATEWAY_DEPLOY_ROLE, path: ROLE_PATH,
      assumedBy: new iam.WebIdentityPrincipal(iamArn(`oidc-provider/${oidc}`), {
        StringEquals: {
          [`${oidc}:aud`]: 'sts.amazonaws.com',
          [`${oidc}:sub`]: `repo:${GITHUB_SUBJECT_REPOSITORY}:environment:${GITHUB_ENVIRONMENT}`,
        },
      }),
      inlinePolicies: {
        DeployGateway: new iam.PolicyDocument({ statements: [
          new iam.PolicyStatement({
            actions: ['cloudformation:CreateChangeSet'], resources: [arn('cloudformation', `stack/${GATEWAY_STACK}/*`)],
            conditions: {
              ArnEquals: { 'cloudformation:RoleArn': execution.roleArn },
              StringLike: { 'cloudformation:ChangeSetName': 'theia-gateway-*' },
            },
          }),
          new iam.PolicyStatement({
            actions: [
              'cloudformation:DescribeChangeSet', 'cloudformation:ExecuteChangeSet',
              'cloudformation:DeleteChangeSet',
            ],
            resources: [arn('cloudformation', `stack/${GATEWAY_STACK}/*`)],
          }),
          new iam.PolicyStatement({
            actions: ['cloudformation:DescribeStacks', 'cloudformation:UpdateTerminationProtection'],
            resources: [arn('cloudformation', `stack/${GATEWAY_STACK}/*`)],
          }),
          new iam.PolicyStatement({
            actions: ['iam:PassRole'], resources: [execution.roleArn],
            conditions: { StringEquals: { 'iam:PassedToService': 'cloudformation.amazonaws.com' } },
          }),
          new iam.PolicyStatement({ actions: ['s3:PutObject'], resources: [artifact] }),
        ] }),
      },
    });
    const exported = (name: string, value: string) =>
      new CfnOutput(this, `${name}Export`, { value, exportName: gatewayExport(name) });
    exported('ApiId', http.ref);
    exported('InstanceId', instance);
    exported('ArtifactBucket', bucket);
    exported('LambdaRuntimeArn', runtime.roleArn);
    new CfnOutput(this, 'DeploymentRoleArn', { value: deploy.roleArn });
    new CfnOutput(this, 'ExecutionRoleArn', { value: execution.roleArn });
  }
}

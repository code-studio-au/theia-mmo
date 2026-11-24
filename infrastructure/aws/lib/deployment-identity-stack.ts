import { Aws, CfnOutput, CfnParameter, Stack, Tags, aws_iam as iam } from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import {
  BUDGET_NAME, DEPLOY_ROLE, EXECUTION_ROLE, GITHUB_ENVIRONMENT, GITHUB_REPOSITORY,
  HOST_PROFILE, HOST_ROLE, HOST_STACK, PROJECT, ROLE_PATH,
} from './names.ts';

type Condition = Record<string, Record<string, string | string[]>>;

function allow(Sid: string, Action: string[], Resource: string | string[], Condition?: Condition) {
  return { Sid, Effect: 'Allow', Action, Resource, ...(Condition && { Condition }) };
}

export class TheiaDeploymentIdentityStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);
    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');
    Tags.of(this).add('ManagedBy', 'CDK');

    const reviewedAmi = new CfnParameter(this, 'ReviewedAmiId', {
      type: 'AWS::EC2::Image::Id',
      description: 'Reviewed Canonical image permitted for the initial Theia host.',
    });
    const ec2 = (resource: string) => `arn:${Aws.PARTITION}:ec2:${Aws.REGION}:${Aws.ACCOUNT_ID}:${resource}`;
    const iamArn = (resource: string) => `arn:${Aws.PARTITION}:iam::${Aws.ACCOUNT_ID}:${resource}`;
    const creates = {
      CreateVpc: 'vpc/*',
      CreateSubnet: 'subnet/*',
      CreateInternetGateway: 'internet-gateway/*',
      CreateRouteTable: 'route-table/*',
      CreateSecurityGroup: 'security-group/*',
      CreateNetworkInterface: 'network-interface/*',
      AllocateAddress: 'elastic-ip/*',
    };
    const network = Object.values(creates).map(ec2);
    const instance = ec2('instance/*');
    const volume = ec2('volume/*');
    const projectInRequest = { 'aws:RequestTag/Project': PROJECT };
    const projectResource = { StringEquals: { 'aws:ResourceTag/Project': PROJECT } };
    const regionOnly = { StringEquals: { 'aws:RequestedRegion': Aws.REGION } };
    const runtime = iamArn(`role${ROLE_PATH}${HOST_ROLE}`);
    const profile = iamArn(`instance-profile${ROLE_PATH}${HOST_PROFILE}`);
    const ssmPolicy = `arn:${Aws.PARTITION}:iam::aws:policy/AmazonSSMManagedInstanceCore`;
    const bootstrapVersion =
      `arn:${Aws.PARTITION}:ssm:${Aws.REGION}:${Aws.ACCOUNT_ID}:parameter/cdk-bootstrap/hnb659fds/version`;

    const execution = new iam.CfnRole(this, 'CloudFormationExecution', {
      roleName: EXECUTION_ROLE,
      path: ROLE_PATH,
      description: 'Manage the single Theia development host and its tagged dependencies.',
      assumeRolePolicyDocument: {
        Version: '2012-10-17',
        Statement: [{ Effect: 'Allow', Principal: { Service: 'cloudformation.amazonaws.com' }, Action: 'sts:AssumeRole' }],
      },
      policies: [{
        policyName: 'TheiaDevelopmentHostResources',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            allow('ReadSydneyHostResources', [
              'DescribeAddresses', 'DescribeAvailabilityZones', 'DescribeImages',
              'DescribeIamInstanceProfileAssociations', 'DescribeInstanceAttribute',
              'DescribeInstanceCreditSpecifications', 'DescribeInstances',
              'DescribeInternetGateways', 'DescribeKeyPairs', 'DescribeLaunchTemplates',
              'DescribeLaunchTemplateVersions', 'DescribeNetworkInterfaces',
              'DescribeRouteTables', 'DescribeSecurityGroupRules', 'DescribeSecurityGroups',
              'DescribeSubnets', 'DescribeTags', 'DescribeVolumes', 'DescribeVpcAttribute',
              'DescribeVpcs',
            ].map((action) => `ec2:${action}`), '*', regionOnly),
            ...Object.entries(creates).map(([action, resource]) =>
              allow(`TagOn${action}`, [`ec2:${action}`], ec2(resource), { StringEquals: projectInRequest })),
            allow('CreateInsideTheiaNetwork', [
              'ec2:CreateSubnet', 'ec2:CreateRouteTable', 'ec2:CreateSecurityGroup',
              'ec2:CreateNetworkInterface',
            ], [ec2('vpc/*'), ec2('subnet/*'), ec2('security-group/*')], projectResource),
            allow('ManageTheiaNetwork', [
              'ModifyVpcAttribute', 'DeleteVpc', 'ModifySubnetAttribute', 'DeleteSubnet',
              'AttachInternetGateway', 'DetachInternetGateway', 'DeleteInternetGateway',
              'CreateRoute', 'ReplaceRoute', 'DeleteRoute', 'DeleteRouteTable',
              'AssociateRouteTable', 'ReplaceRouteTableAssociation', 'DisassociateRouteTable',
              'AuthorizeSecurityGroupEgress', 'RevokeSecurityGroupEgress', 'DeleteSecurityGroup',
              'ModifyNetworkInterfaceAttribute', 'DeleteNetworkInterface',
              'AssociateAddress', 'DisassociateAddress', 'ReleaseAddress',
            ].map((action) => `ec2:${action}`), [...network, instance], projectResource),
            allow('LaunchTheiaInstance', ['ec2:RunInstances'], instance, {
              StringEquals: {
                ...projectInRequest,
                'ec2:InstanceType': 't3a.medium',
                'ec2:MetadataHttpTokens': 'required',
              },
            }),
            allow('UseReviewedImage', ['ec2:RunInstances'],
              `arn:${Aws.PARTITION}:ec2:${Aws.REGION}::image/${reviewedAmi.valueAsString}`,
            ),
            allow('UseEncryptedGp3Root', ['ec2:RunInstances'], volume, {
              Bool: { 'ec2:Encrypted': 'true' },
              StringEquals: { 'ec2:VolumeType': 'gp3' },
              NumericEquals: { 'ec2:VolumeSize': '40' },
            }),
            allow('LaunchInTheiaNetwork', ['ec2:RunInstances'], [
              ec2('subnet/*'), ec2('network-interface/*'), ec2('security-group/*'),
            ], projectResource),
            allow('ManageTheiaInstance', [
              'ec2:StartInstances', 'ec2:StopInstances', 'ec2:TerminateInstances',
              'ec2:ModifyInstanceAttribute', 'ec2:ModifyInstanceCreditSpecification',
              'ec2:ModifyInstanceMetadataOptions', 'ec2:AssociateIamInstanceProfile',
              'ec2:DisassociateIamInstanceProfile', 'ec2:ReplaceIamInstanceProfileAssociation',
            ], instance, projectResource),
            allow('TagResourcesAtCreation', ['ec2:CreateTags'], [...network, instance, volume], {
              StringEquals: { ...projectInRequest, 'ec2:CreateAction': [...Object.keys(creates), 'RunInstances'] },
            }),
            allow('UpdateTheiaTags', ['ec2:CreateTags'], [...network, instance, volume], {
              ...projectResource,
              StringEqualsIfExists: projectInRequest,
            }),
            allow('ManageRuntimeRole', [
              'iam:CreateRole', 'iam:GetRole', 'iam:DeleteRole',
              'iam:TagRole', 'iam:UntagRole', 'iam:ListAttachedRolePolicies', 'iam:ListRolePolicies',
            ], runtime),
            allow('ManageOnlyRuntimeSsmPolicy', ['iam:AttachRolePolicy', 'iam:DetachRolePolicy'], runtime,
              { ArnEquals: { 'iam:PolicyARN': ssmPolicy } }),
            allow('ManageRuntimeProfile', [
              'iam:CreateInstanceProfile', 'iam:GetInstanceProfile', 'iam:DeleteInstanceProfile',
              'iam:AddRoleToInstanceProfile', 'iam:RemoveRoleFromInstanceProfile',
            ], profile),
            allow('PassOnlyTheiaRuntimeRole', ['iam:PassRole'], runtime,
              { StringEquals: { 'iam:PassedToService': 'ec2.amazonaws.com' } }),
            allow('ReadBootstrapVersion', ['ssm:GetParameter', 'ssm:GetParameters'], bootstrapVersion),
            allow('ManageTheiaBudget', [
              'budgets:ModifyBudget', 'budgets:ViewBudget', 'budgets:TagResource',
              'budgets:UntagResource', 'budgets:ListTagsForResource',
            ], `arn:${Aws.PARTITION}:budgets::${Aws.ACCOUNT_ID}:budget/${BUDGET_NAME}`),
          ],
        },
      }],
    });

    const hostStack = `arn:${Aws.PARTITION}:cloudformation:${Aws.REGION}:${Aws.ACCOUNT_ID}:stack/${HOST_STACK}/*`;
    const oidc = 'token.actions.githubusercontent.com';
    const deploy = new iam.CfnRole(this, 'GitHubDeployer', {
      roleName: DEPLOY_ROLE,
      path: ROLE_PATH,
      description: 'Deploy only the reviewed Theia development stack from its protected GitHub environment.',
      maxSessionDuration: 3600,
      assumeRolePolicyDocument: {
        Version: '2012-10-17',
        Statement: [{
          Effect: 'Allow',
          Principal: { Federated: iamArn(`oidc-provider/${oidc}`) },
          Action: 'sts:AssumeRoleWithWebIdentity',
          Condition: {
            StringEquals: {
              [`${oidc}:aud`]: 'sts.amazonaws.com',
              [`${oidc}:sub`]: `repo:${GITHUB_REPOSITORY}:environment:${GITHUB_ENVIRONMENT}`,
            },
          },
        }],
      },
      policies: [{
        policyName: 'DeployTheiaDevelopmentHost',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            allow('PrepareOnlyTheiaStack', [
              'cloudformation:CreateChangeSet', 'cloudformation:CreateStack', 'cloudformation:UpdateStack',
            ], hostStack, { ArnEquals: { 'cloudformation:RoleArn': execution.attrArn } }),
            allow('InspectAndExecuteTheiaChanges', [
              'cloudformation:DescribeChangeSet', 'cloudformation:DescribeStacks',
              'cloudformation:DescribeStackEvents', 'cloudformation:DescribeStackResources',
              'cloudformation:GetTemplate', 'cloudformation:ListStackResources',
              'cloudformation:ExecuteChangeSet', 'cloudformation:DeleteChangeSet',
              'cloudformation:UpdateTerminationProtection',
            ], hostStack),
            allow('PassOnlyTheiaExecutor', ['iam:PassRole'], execution.attrArn,
              { StringEquals: { 'iam:PassedToService': 'cloudformation.amazonaws.com' } }),
            allow('ValidateSydneyStack', ['cloudformation:ValidateTemplate'], '*', regionOnly),
            allow('ReadBootstrapVersion', ['ssm:GetParameter', 'ssm:GetParameters'], bootstrapVersion),
          ],
        },
      }],
    });
    new CfnOutput(this, 'DeploymentRoleArn', { value: deploy.attrArn });
    new CfnOutput(this, 'ExecutionRoleArn', { value: execution.attrArn });
  }
}

import {
  Aws, CfnOutput, CfnParameter, Stack, Tags,
  aws_budgets as budgets, aws_ec2 as ec2, aws_iam as iam,
} from 'aws-cdk-lib';
import type { StackProps } from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { BUDGET_NAME, HOST_PROFILE, HOST_ROLE, PROJECT, ROLE_PATH } from './names.ts';

export class TheiaHostStack extends Stack {
  constructor(scope: Construct, id: string, props: StackProps) {
    super(scope, id, props);

    Tags.of(this).add('Project', PROJECT);
    Tags.of(this).add('Environment', 'development');
    Tags.of(this).add('ManagedBy', 'CDK');

    // Keep the exact reviewed image stable across unrelated stack updates.
    const ami = new CfnParameter(this, 'UbuntuAmiId', {
      type: 'AWS::EC2::Image::Id',
      description: 'Reviewed Canonical Ubuntu 24.04 LTS AMD64 gp3 AMI in Sydney.',
    });
    const zone = new CfnParameter(this, 'HostAvailabilityZone', {
      type: 'AWS::EC2::AvailabilityZone::Name',
      description: 'Reviewed Sydney availability zone offering t3a.medium.',
    });
    const amount = new CfnParameter(this, 'MonthlyBudgetUsd', {
      type: 'Number',
      minValue: 1,
      description: 'Approved monthly budget for tagged Theia development costs in USD.',
    });
    const email = new CfnParameter(this, 'BudgetAlertEmail', {
      type: 'String',
      noEcho: true,
      allowedPattern: '^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$',
      description: 'Recipient for Theia development budget alerts.',
    });

    const vpc = new ec2.CfnVPC(this, 'Vpc', {
      cidrBlock: '10.84.0.0/24',
      enableDnsSupport: true,
      enableDnsHostnames: true,
    });
    const subnet = new ec2.CfnSubnet(this, 'HostSubnet', {
      vpcId: vpc.ref,
      availabilityZone: zone.valueAsString,
      cidrBlock: '10.84.0.0/26',
      mapPublicIpOnLaunch: false,
    });
    const gateway = new ec2.CfnInternetGateway(this, 'InternetGateway');
    const attachment = new ec2.CfnVPCGatewayAttachment(this, 'InternetGatewayAttachment', {
      vpcId: vpc.ref,
      internetGatewayId: gateway.ref,
    });
    const routes = new ec2.CfnRouteTable(this, 'HostRouteTable', { vpcId: vpc.ref });
    const internetRoute = new ec2.CfnRoute(this, 'InternetRoute', {
      routeTableId: routes.ref,
      destinationCidrBlock: '0.0.0.0/0',
      gatewayId: gateway.ref,
    });
    internetRoute.addResourceDependency(attachment);
    const routeTableAssociation = new ec2.CfnSubnetRouteTableAssociation(this, 'HostSubnetRoutes', {
      subnetId: subnet.ref,
      routeTableId: routes.ref,
    });

    const firewall = new ec2.CfnSecurityGroup(this, 'HostFirewall', {
      vpcId: vpc.ref,
      groupDescription: 'Theia development host; outbound HTTPS for SSM and approved downloads',
      securityGroupIngress: [],
      securityGroupEgress: [{
        ipProtocol: 'tcp', fromPort: 443, toPort: 443, cidrIp: '0.0.0.0/0',
        description: 'AWS Systems Manager and approved HTTPS endpoints',
      }],
    });
    const role = new iam.Role(this, 'HostRole', {
      roleName: HOST_ROLE,
      path: ROLE_PATH,
      assumedBy: new iam.ServicePrincipal('ec2.amazonaws.com'),
      managedPolicies: [iam.ManagedPolicy.fromAwsManagedPolicyName('AmazonSSMManagedInstanceCore')],
    });
    const profile = new iam.CfnInstanceProfile(this, 'HostProfile', {
      instanceProfileName: HOST_PROFILE, path: ROLE_PATH, roles: [role.roleName],
    });
    const hostInterface = new ec2.CfnNetworkInterface(this, 'HostNetworkInterface', {
      subnetId: subnet.ref,
      groupSet: [firewall.attrGroupId],
    });

    const host = new ec2.CfnInstance(this, 'Host', {
      imageId: ami.valueAsString,
      instanceType: 't3a.medium',
      creditSpecification: { cpuCredits: 'standard' },
      networkInterfaces: [{ deviceIndex: '0', networkInterfaceId: hostInterface.ref, deleteOnTermination: false }],
      iamInstanceProfile: profile.ref,
      metadataOptions: { httpEndpoint: 'enabled', httpTokens: 'required', httpPutResponseHopLimit: 1 },
      disableApiTermination: true,
      instanceInitiatedShutdownBehavior: 'stop',
      propagateTagsToVolumeOnCreation: true,
      blockDeviceMappings: [{
        deviceName: '/dev/sda1',
        ebs: { encrypted: true, volumeType: 'gp3', volumeSize: 40, deleteOnTermination: false },
      }],
    });
    Tags.of(host).add('Name', 'theia-development');

    const address = new ec2.CfnEIP(this, 'HostAddress', { domain: 'vpc' });
    const association = new ec2.CfnEIPAssociation(this, 'HostAddressAssociation', {
      allocationId: address.attrAllocationId,
      networkInterfaceId: hostInterface.ref,
    });
    association.addResourceDependency(host);
    association.addResourceDependency(internetRoute);
    association.addResourceDependency(routeTableAssociation);

    new budgets.CfnBudget(this, 'MonthlyBudget', {
      budget: {
        budgetName: BUDGET_NAME,
        budgetType: 'COST',
        timeUnit: 'MONTHLY',
        budgetLimit: { amount: amount.valueAsNumber, unit: 'USD' },
        filterExpression: { tags: { key: 'user:Project', values: [PROJECT] } },
        metrics: ['NetUnblendedCost'],
      },
      notificationsWithSubscribers: [50, 80, 100].map((threshold) => ({
        notification: {
          comparisonOperator: 'GREATER_THAN', notificationType: 'ACTUAL',
          threshold, thresholdType: 'PERCENTAGE',
        },
        subscribers: [{ subscriptionType: 'EMAIL', address: email.valueAsString }],
      })),
    });

    new CfnOutput(this, 'HostInstanceId', { value: host.ref });
    new CfnOutput(this, 'HostPublicIpv4', { value: address.ref });
    new CfnOutput(this, 'SessionManagerCommand', {
      value: `aws ssm start-session --region ${Aws.REGION} --target ${host.ref}`,
    });
  }
}

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { App } from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import { TheiaHostStack } from '../lib/host-stack.ts';
import { REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
const stack = new TheiaHostStack(app, 'TestTheiaHost', {
  env: { account: '111111111111', region: REGION },
  terminationProtection: true,
});
const template = Template.fromStack(stack);

test('the test host has durable storage and limits CPU credit billing', () => {
  template.resourceCountIs('AWS::EC2::Instance', 1);
  template.hasResourceProperties('AWS::EC2::Instance', {
    ImageId: { Ref: 'UbuntuAmiId' },
    InstanceType: 't3a.medium',
    BlockDeviceMappings: [{
      DeviceName: '/dev/sda1',
      Ebs: { DeleteOnTermination: false, Encrypted: true, VolumeType: 'gp3', VolumeSize: 40 },
    }],
    CreditSpecification: { CPUCredits: 'standard' },
    DisableApiTermination: true,
    InstanceInitiatedShutdownBehavior: 'stop',
    PropagateTagsToVolumeOnCreation: true,
    MetadataOptions: { HttpEndpoint: 'enabled', HttpTokens: 'required', HttpPutResponseHopLimit: 1 },
    KeyName: Match.absent(),
  });
  assert.equal(stack.terminationProtection, true);
  template.hasResourceProperties('AWS::EC2::Subnet', {
    AvailabilityZone: { Ref: 'HostAvailabilityZone' },
    MapPublicIpOnLaunch: false,
  });
});

test('the public address has no inbound access and the instance role only provides SSM management', () => {
  template.resourceCountIs('AWS::EC2::EIP', 1);
  template.resourceCountIs('AWS::EC2::EIPAssociation', 1);
  template.hasResourceProperties('AWS::EC2::SecurityGroup', {
    SecurityGroupIngress: [],
    SecurityGroupEgress: [{
      IpProtocol: 'tcp', CidrIp: '0.0.0.0/0', FromPort: 443, ToPort: 443,
      Description: 'AWS Systems Manager and approved HTTPS endpoints',
    }],
  });
  template.resourceCountIs('AWS::EC2::SecurityGroupIngress', 0);
  template.hasResourceProperties('AWS::IAM::Role', {
    ManagedPolicyArns: [{
      'Fn::Join': ['', ['arn:', { Ref: 'AWS::Partition' },
        ':iam::aws:policy/AmazonSSMManagedInstanceCore']],
    }],
    Policies: Match.absent(),
  });
  for (const type of ['AWS::EC2::NatGateway', 'AWS::RDS::DBInstance',
    'AWS::ElasticLoadBalancingV2::LoadBalancer', 'AWS::S3::Bucket']) {
    template.resourceCountIs(type, 0);
  }
});

test('budget alerts watch the project tag and require an explicit amount and recipient', () => {
  template.hasResourceProperties('AWS::Budgets::Budget', {
    Budget: {
      BudgetLimit: { Amount: { Ref: 'MonthlyBudgetUsd' }, Unit: 'USD' },
      FilterExpression: { Tags: { Key: 'user:Project', Values: ['TheiaMMO'] } },
      TimeUnit: 'MONTHLY',
    },
    NotificationsWithSubscribers: [50, 80, 100].map((threshold) => ({
      Notification: {
        ComparisonOperator: 'GREATER_THAN', NotificationType: 'ACTUAL',
        Threshold: threshold, ThresholdType: 'PERCENTAGE',
      },
      Subscribers: [{ SubscriptionType: 'EMAIL', Address: { Ref: 'BudgetAlertEmail' } }],
    })),
  });
  template.hasParameter('MonthlyBudgetUsd', { Type: 'Number', Default: Match.absent() });
  template.hasParameter('BudgetAlertEmail', { Type: 'String', Default: Match.absent() });
});

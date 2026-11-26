import { App, BootstraplessSynthesizer, CfnParameter, Stack, aws_ec2 as ec2 } from 'aws-cdk-lib';
import { REGION } from '../lib/names.ts';

// Roll back player access independently of the persistent host and database.
const app = new App({ analyticsReporting: false });
const stack = new Stack(app, 'TheiaDevelopmentRealmIngress', {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
});
const group = new CfnParameter(stack, 'HostSecurityGroupId', { type: 'AWS::EC2::SecurityGroup::Id' });
new ec2.CfnSecurityGroupIngress(stack, 'AccountCertificateValidation', {
  groupId: group.valueAsString, ipProtocol: 'tcp', fromPort: 80, toPort: 80,
  cidrIp: '0.0.0.0/0', description: 'ACME HTTP challenge for the public Theia account API certificate',
});
new ec2.CfnSecurityGroupIngress(stack, 'AccountHttps', {
  groupId: group.valueAsString, ipProtocol: 'tcp', fromPort: 443, toPort: 443,
  cidrIp: '0.0.0.0/0', description: 'Theia tester authentication and character API over HTTPS',
});
new ec2.CfnSecurityGroupIngress(stack, 'EncryptedGame', {
  groupId: group.valueAsString, ipProtocol: 'udp', fromPort: 5000, toPort: 5000,
  cidrIp: '0.0.0.0/0', description: 'Theia encrypted Renet game traffic requires an admission ticket',
});

import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { RealmGatewayStack } from '../lib/realm-gateway-stack.ts';
import { REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new RealmGatewayStack(app, 'TheiaDevelopmentRealmGateway', {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

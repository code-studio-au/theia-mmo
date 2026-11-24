import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { RealmGatewayStack } from '../lib/realm-gateway-stack.ts';
import { GATEWAY_STACK, REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new RealmGatewayStack(app, GATEWAY_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

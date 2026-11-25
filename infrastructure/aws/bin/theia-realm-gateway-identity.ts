import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { RealmGatewayIdentityStack } from '../lib/realm-gateway-identity-stack.ts';
import { GATEWAY_IDENTITY_STACK, REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new RealmGatewayIdentityStack(app, GATEWAY_IDENTITY_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

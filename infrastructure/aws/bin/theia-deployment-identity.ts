import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { TheiaDeploymentIdentityStack } from '../lib/deployment-identity-stack.ts';
import { IDENTITY_STACK, REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new TheiaDeploymentIdentityStack(app, IDENTITY_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

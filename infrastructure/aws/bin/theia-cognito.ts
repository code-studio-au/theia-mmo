import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { COGNITO_STACK, TheiaCognitoStack } from '../lib/cognito-stack.ts';
import { REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new TheiaCognitoStack(app, COGNITO_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

import { App, BootstraplessSynthesizer } from 'aws-cdk-lib';
import { NATIVE_COGNITO_STACK, TheiaCognitoNativeStack } from '../lib/cognito-native-stack.ts';
import { REGION } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new TheiaCognitoNativeStack(app, NATIVE_COGNITO_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new BootstraplessSynthesizer(),
  terminationProtection: true,
});

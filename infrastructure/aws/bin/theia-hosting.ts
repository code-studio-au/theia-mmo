import { App, DefaultStackSynthesizer } from 'aws-cdk-lib';
import { TheiaHostStack } from '../lib/host-stack.ts';
import { DEPLOY_ROLE, EXECUTION_ROLE, HOST_STACK, REGION, ROLE_PATH } from '../lib/names.ts';

const app = new App({ analyticsReporting: false });
new TheiaHostStack(app, HOST_STACK, {
  env: { region: REGION, account: process.env.CDK_DEFAULT_ACCOUNT },
  synthesizer: new DefaultStackSynthesizer({
    deployRoleArn: 'arn:aws:iam::${AWS::AccountId}:role' + ROLE_PATH + DEPLOY_ROLE,
    cloudFormationExecutionRole: 'arn:aws:iam::${AWS::AccountId}:role' + ROLE_PATH + EXECUTION_ROLE,
  }),
  terminationProtection: true,
});

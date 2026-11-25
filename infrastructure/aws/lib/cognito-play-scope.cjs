// The installed public launcher still receives no AWS credentials or IAM role.
const nativeSources = new Set([
  'TokenGeneration_Authentication',
  'TokenGeneration_NewPasswordChallenge',
  'TokenGeneration_RefreshTokens',
]);

exports.handler = async event => {
  const groups = event.request?.groupConfiguration?.groupsToOverride;
  if (!process.env.THEIA_COGNITO_CLIENT
      || event.callerContext?.clientId !== process.env.THEIA_COGNITO_CLIENT
      || !nativeSources.has(event.triggerSource)
      || !Array.isArray(groups) || !groups.includes('theia-testers')) return event;

  const scope = 'theia-development/play';
  if (event.request?.scopes?.includes(scope)) return event;
  const response = event.response ??= {};
  const details = response.claimsAndScopeOverrideDetails ??= {};
  const access = details.accessTokenGeneration ??= {};
  access.scopesToAdd = [...new Set([...(access.scopesToAdd ?? []), scope])];
  return event;
};

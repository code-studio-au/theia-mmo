const assert = require('node:assert/strict');

const initial = { pool: 'ap-southeast-2_Example', client: 'client1', username: 'tester', value: 'secret-password' };
const token = `e30.${Buffer.from(JSON.stringify({ iat: 1700000000, exp: 2050000000 })).toString('base64url')}.c2ln`;

const scenario = process.env.THEIA_COGNITO_TEST;
global.fetch = async (endpoint, options) => {
  assert.equal(new URL(endpoint).origin, 'https://cognito-idp.ap-southeast-2.amazonaws.com');
  const body = JSON.parse(options.body);
  assert.equal(body.ClientId, initial.client);
  const respond = (data, status = 200) => new Response(JSON.stringify(data), { status });
  const operation = options.headers['X-Amz-Target'].split('.').pop();
  if (operation === 'InitiateAuth') {
    assert.equal(body.AuthFlow, 'USER_SRP_AUTH');
    assert.equal(body.AuthParameters.USERNAME, initial.username);
    assert.match(body.AuthParameters.SRP_A, /^[a-f0-9]+$/i);
    assert.equal(options.body.includes(initial.value), false);
    if (scenario === 'wait') {
      await new Promise(resolve => setTimeout(resolve, 6000));
    }
    if (scenario === 'invalid') {
      return respond({ __type: 'NotAuthorizedException', message: 'private Cognito response' }, 400);
    }
    return respond({ ChallengeName: 'PASSWORD_VERIFIER', ChallengeParameters: {
      USERNAME: initial.username, USER_ID_FOR_SRP: initial.username, SRP_B: '02', SALT: '01', SECRET_BLOCK: 'AA==',
    } });
  }

  assert.equal(operation, 'RespondToAuthChallenge');
  assert.equal(body.ChallengeResponses.USERNAME, initial.username);
  if (body.ChallengeName === 'PASSWORD_VERIFIER') {
    assert.match(body.ChallengeResponses.PASSWORD_CLAIM_SIGNATURE, /^[A-Za-z0-9+/]+=*$/);
    assert.equal(options.body.includes(initial.value), false);
    if (scenario === 'setup') return respond({ ChallengeName: 'MFA_SETUP', Session: 'setup' });
    if (scenario === 'challenges') {
      return respond({ ChallengeName: 'NEW_PASSWORD_REQUIRED', Session: 'new-password', ChallengeParameters: {
        userAttributes: '{}', requiredAttributes: '[]',
      } });
    }
  } else if (body.ChallengeName === 'NEW_PASSWORD_REQUIRED') {
    assert.equal(body.Session, 'new-password');
    assert.equal(body.ChallengeResponses.NEW_PASSWORD, 'new-secret-password');
    return respond({ ChallengeName: 'SOFTWARE_TOKEN_MFA', Session: 'mfa' });
  } else {
    assert.equal(body.ChallengeName, 'SOFTWARE_TOKEN_MFA');
    assert.equal(body.Session, 'mfa');
    assert.equal(body.ChallengeResponses.SOFTWARE_TOKEN_MFA_CODE, '123456');
  }
  return respond({ AuthenticationResult: { AccessToken: token, IdToken: token, RefreshToken: 'private refresh' } });
};

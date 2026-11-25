// Private line protocol with the GTK launcher's worker. Cognito handles SRP.
const { createInterface } = require('node:readline');
const { AuthenticationDetails, CognitoUser, CognitoUserPool } = require('amazon-cognito-identity-js');

const input = createInterface({ input: process.stdin, crlfDelay: Infinity });
const errors = new Set(['NotAuthorizedException', 'UserNotFoundException', 'PasswordResetRequiredException',
  'UserNotConfirmedException', 'CodeMismatchException', 'ExpiredCodeException', 'InvalidPasswordException',
  'TooManyRequestsException', 'LimitExceededException']);
let done = false;
let answer;
let started = false;

function send(message) {
  if (!done) process.stdout.write(JSON.stringify(message) + '\n');
}

function end(message) {
  if (done) return;
  send(message);
  done = true;
  input.close();
}

function failure(code) {
  end({ error: errors.has(code) ? code : 'Unavailable' });
}

function challenge(kind, next) {
  if (done) return;
  answer = next;
  send({ challenge: kind });
}

function signIn({ pool, client, username, value }) {
  if (!/^ap-southeast-2_[A-Za-z0-9]+$/.test(pool) || !/^[A-Za-z0-9]{1,128}$/.test(client)
      || typeof username !== 'string' || !username.length || username.length > 128
      || typeof value !== 'string' || !value.length || value.length > 256) {
    failure('NotAuthorizedException');
    return;
  }
  // Never persist Cognito sessions or use browser-local storage.
  const storage = { getItem: () => null, setItem: () => {}, removeItem: () => {}, clear: () => {} };
  const userPool = new CognitoUserPool({ UserPoolId: pool, ClientId: client, Storage: storage,
    AdvancedSecurityDataCollectionFlag: false });
  const user = new CognitoUser({ Username: username, Pool: userPool, Storage: storage });
  const callbacks = {
    onSuccess: session => end({ access_token: session.getAccessToken().getJwtToken() }),
    onFailure: error => failure(error?.code || error?.name),
    newPasswordRequired: (_attributes, required) => {
      if (required?.length) {
        end({ error: 'BrowserRequired' });
        return;
      }
      challenge('new_password', password => user.completeNewPasswordChallenge(password, {}, callbacks));
    },
    mfaRequired: () => challenge('sms_code', code => user.sendMFACode(code, callbacks, 'SMS_MFA')),
    totpRequired: () => challenge('totp_code', code => user.sendMFACode(code, callbacks, 'SOFTWARE_TOKEN_MFA')),
    customChallenge: () => end({ error: 'BrowserRequired' }),
    mfaSetup: () => end({ error: 'BrowserRequired' }),
    selectMFAType: () => end({ error: 'BrowserRequired' }),
  };
  user.setAuthenticationFlowType('USER_SRP_AUTH');
  user.authenticateUser(new AuthenticationDetails({ Username: username, Password: value }), callbacks);
}

input.on('line', line => {
  if (done) return;
  try {
    if (line.length > 8192) throw new Error();
    const data = JSON.parse(line);
    if (!started) {
      started = true;
      signIn(data);
    } else if (answer && typeof data.value === 'string' && data.value.length && data.value.length <= 256) {
      const next = answer;
      answer = undefined;
      next(data.value);
    } else {
      failure('Unavailable');
    }
  } catch {
    failure('Unavailable');
  }
});

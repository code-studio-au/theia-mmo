const { appendFileSync, readdirSync, readFileSync } = require('node:fs');
const { join } = require('node:path');
const { packages } = require('./package-lock.json');

const notices = 'dist/cognito-sign-in.cjs.LEGAL.txt';
for (const [directory, metadata] of Object.entries(packages)) {
  if (!directory || metadata.dev) continue;
  for (const name of readdirSync(directory).filter(name => /^(licen[sc]e|notice|copyright)/i.test(name))) {
    const contents = readFileSync(join(directory, name), 'utf8');
    appendFileSync(notices, `\n\n${directory}/${name}\n\n${contents}\n`);
  }
}

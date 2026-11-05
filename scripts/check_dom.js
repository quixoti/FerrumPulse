const fs = require('fs');
const html = fs.readFileSync('f:/FerrumPulse/ui/index.html', 'utf8');
const appJs = fs.readFileSync('f:/FerrumPulse/ui/src/app.js', 'utf8');

const regex = /getElementById\(['"]([^'"]+)['"]\)/g;
let match;
const missing = [];
while ((match = regex.exec(appJs)) !== null) {
  const id = match[1];
  if (!html.includes(`id="${id}"`)) {
    missing.push(id);
  }
}
console.log('MISSING IDS:', missing);


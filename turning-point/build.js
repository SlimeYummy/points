const fs = require('fs');

fs.mkdirSync('./bin/native', { recursive: true });
fs.copyFileSync('./index.js', './bin/native/native.js');
fs.copyFileSync('./index.d.ts', './bin/native/native.d.ts');

fs.mkdirSync('./src/native', { recursive: true });
fs.copyFileSync('./index.d.ts', './src/native/native.d.ts');
fs.copyFileSync('./index.js', './src/native/native.js');

fs.unlinkSync('./index.js');
fs.unlinkSync('./index.d.ts');

for (const file of fs.readdirSync('.')) {
    if (file.startsWith('turning-point-node')) {
        fs.copyFileSync(file, `./bin/native/${file}`);
        fs.copyFileSync(file, `./src/native/${file}`);
        fs.unlinkSync(file);
    }
}

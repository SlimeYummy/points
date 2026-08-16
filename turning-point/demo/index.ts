import { Resource, Script, Zone } from '../src';
import './player';
import './npc';

new Zone('Zone.Demo', {
    name: 'Demo',
    files: 'Zones/TestZone.*',
    view_file: 'TestZone.unity',
});

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/demo-template`);
Script.write(`${__dirname}/../../turning-point-wasm`, `${__dirname}/../../test-tmp/demo-template`);
console.log('\nGenerate templates done\n');

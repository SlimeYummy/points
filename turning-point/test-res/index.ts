import { Resource } from '../src';
import './instance';
import './template';
import './verify';

const extra = ['Aaa', 'Bbb', 'Ccc', 'Ddd', 'Eee', 'Fff', 'Ggg', 'Xxx', 'Yyy', 'Zzz', 'Empty'];

declare const __dirname: string;
console.log('');
Resource.write(`${__dirname}/../../test-tmp/test-template`, extra);

console.log('\nGenerate templates done\n');

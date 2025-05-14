import { Resource } from '../src/resource';
import './instance';
import './template';
import './verify';

const extra = ['Aaa', 'Bbb', 'Ccc', 'Ddd', 'Eee', 'Fff', 'Ggg', 'Xxx', 'Yyy', 'Zzz', 'Empty'];

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/test-template`, extra);

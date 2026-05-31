// @ts-check

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
	{
		ignores: ['bin/**', 'node_modules/**', 'prebuilt/**', 'demo/**', 'test-case/**', 'test-res/**', '*.js'],
	},
	eslint.configs.recommended,
	tseslint.configs.recommended,
	{
		files: ['src/**/*.ts'],
	},
);

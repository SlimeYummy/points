import process from 'node:process';

export const FPS = 60;
export const LOGIC_FPS = 30;
export const LOGIC_SPF = 1.0 / LOGIC_FPS;
export const ENABLE_TIME_WARNING = true;

export const MAX_NAME_LEN = 48;
export const MAX_ENTRY_PLUS = 3;

export const INPUT_ASSET: string = process.env['INPUT_ASSET'] || './asset';
export const OUTPUT_ASSET: string = process.env['OUTPUT_ASSET'] || './out';
export const FORCE_GEN = process.env['FORCE_GEN'] == '1' || process.env['FORCE_GEN'] == 'true';

// const program = new Command();
// program.option('-i, --input-asset', 'Raw asset path');
// program.option('-o, --output-asset', 'Force a rebuild of all assets and templates');
// program.option('-f, --force', 'Force a rebuild of all assets and templates');

// program.parse(process.argv);
// const options = program.opts();

// export const INPUT_ASSET = options['input-asset'] || process.env['INPUT_ASSET'] || './asset';
// export const OUTPUT_ASSET = options['output-asset'] || process.env['OUTPUT_ASSET'] || './out';
// export const FORCE_GEN = options['force'] || process.env['FORCE_GEN'] || false;

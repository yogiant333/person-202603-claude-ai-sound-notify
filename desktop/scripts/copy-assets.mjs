import { cp, rm, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const srcDir = path.resolve(here, '..', '..', 'web');
const destDir = path.resolve(here, '..', 'dist');

await rm(destDir, { recursive: true, force: true });
await mkdir(destDir, { recursive: true });
await cp(srcDir, destDir, { recursive: true });
console.log(`[copy-assets] ${srcDir} -> ${destDir}`);

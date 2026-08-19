#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rustRoot = resolve(scriptDir, '..');
const cocos4Root = resolve(rustRoot, '..', 'cocos4');
const moduleNameMapper = JSON.stringify({
  '^cc.decorator$': '<rootDir>/cocos/core/data/decorators/index.ts',
});

function runStep(label, command, args, cwd) {
  const started = process.hrtime.bigint();
  const result = spawnSync(command, args, {
    cwd,
    stdio: 'pipe',
    encoding: 'utf8',
  });
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1_000_000;
  return {
    label,
    command: [command, ...args].join(' '),
    cwd,
    status: result.status,
    elapsedMs,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

function printStep(step) {
  console.log(`\n## ${step.label}`);
  console.log(`cwd=${step.cwd}`);
  console.log(`command=${step.command}`);
  console.log(`status=${step.status}`);
  console.log(`elapsed_ms=${step.elapsedMs.toFixed(3)}`);
  if (step.stdout.trim()) {
    console.log('\nstdout:');
    console.log(step.stdout.trim());
  }
  if (step.stderr.trim()) {
    console.log('\nstderr:');
    console.log(step.stderr.trim());
  }
}

if (!existsSync(resolve(cocos4Root, 'package.json'))) {
  console.error(`missing cocos4 checkout: ${cocos4Root}`);
  process.exit(2);
}

if (!existsSync(resolve(cocos4Root, 'node_modules'))) {
  console.error(`missing ${resolve(cocos4Root, 'node_modules')}`);
  console.error('run this first: (cd ../cocos4 && npm ci --ignore-scripts)');
  process.exit(2);
}

const steps = [
  runStep('cocos4 build debug infos', 'npm', ['run', 'build:debug-infos'], cocos4Root),
  runStep(
    'cocos4 Vec3 Jest parity subset',
    'npx',
    [
      'jest',
      'tests/value-types/vec3.test.ts',
      '--runInBand',
      '--no-cache',
      `--moduleNameMapper=${moduleNameMapper}`,
    ],
    cocos4Root,
  ),
  runStep('cocos4-rust Vec3 tests', 'cargo', ['test', 'vec3'], rustRoot),
];

for (const step of steps) {
  printStep(step);
  if (step.status !== 0) {
    process.exit(step.status ?? 1);
  }
}

const original = steps[1];
const rust = steps[2];
console.log('\n## summary');
console.log('| target | command | status | elapsed ms |');
console.log('|---|---|---:|---:|');
console.log(`| cocos4 original Vec3 Jest | \`${original.command}\` | ${original.status} | ${original.elapsedMs.toFixed(3)} |`);
console.log(`| cocos4-rust Vec3 tests | \`${rust.command}\` | ${rust.status} | ${rust.elapsedMs.toFixed(3)} |`);

"use strict";

const { existsSync } = require("node:fs");
const { join } = require("node:path");

function linuxLibc() {
  const report = process.report?.getReport();
  return report?.header?.glibcVersionRuntime ? "gnu" : "musl";
}

function binarySuffix() {
  const { arch, platform } = process;
  const suffixes = {
    darwin: `darwin-${arch}`,
    linux: `linux-${arch}-${linuxLibc()}`,
    win32: `win32-${arch}-msvc`,
  };
  const suffix = suffixes[platform];
  if (!suffix) {
    throw new Error(`celestial-js does not support ${platform}-${arch}`);
  }
  return suffix;
}

const binaryPath = join(__dirname, `index.${binarySuffix()}.node`);
if (!existsSync(binaryPath)) {
  throw new Error(`celestial-js native addon is missing: ${binaryPath}. Run "npm run build".`);
}

module.exports = require(binaryPath);

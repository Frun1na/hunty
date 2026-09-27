#!/usr/bin/env node
/**
 * Fast syntax-only parse check for TypeScript / TSX files.
 *
 * Used by lint-staged in the pre-commit hook to catch files that no longer
 * parse (e.g. literal "\n" escapes pasted into source, merge-conflict debris)
 * before they are committed. It only runs the TypeScript parser — no type
 * checking and no module resolution — so it takes milliseconds per file.
 *
 * Usage: node scripts/check-ts-parse.mjs <file> [file...]
 * Exits 1 and prints file:line:col diagnostics if any file fails to parse.
 */
import { readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import ts from "typescript";

const TS_FILE = /\.(c|m)?tsx?$/;

/**
 * Parses each file and returns one entry per syntax error.
 * @param {string[]} files
 * @returns {{ file: string, line: number, column: number, message: string }[]}
 */
export function findParseErrors(files) {
  const errors = [];
  const readable = [];

  for (const file of files) {
    if (!TS_FILE.test(file)) continue;
    try {
      readFileSync(file);
      readable.push(file);
    } catch (error) {
      errors.push({ file, line: 0, column: 0, message: `Cannot read file: ${error.message}` });
    }
  }

  if (readable.length === 0) return errors;

  // noResolve + noLib keep the program to exactly the given files, and we only
  // ask for syntactic diagnostics, so no type checking ever runs.
  const program = ts.createProgram(readable, {
    noResolve: true,
    noLib: true,
    types: [],
    allowJs: false,
    jsx: ts.JsxEmit.Preserve,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
  });

  for (const file of readable) {
    const sourceFile = program.getSourceFile(file);
    if (!sourceFile) {
      errors.push({ file, line: 0, column: 0, message: "Could not be parsed" });
      continue;
    }

    for (const diagnostic of program.getSyntacticDiagnostics(sourceFile)) {
      const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n");
      if (diagnostic.start !== undefined) {
        const { line, character } = sourceFile.getLineAndCharacterOfPosition(diagnostic.start);
        errors.push({ file, line: line + 1, column: character + 1, message });
      } else {
        errors.push({ file, line: 0, column: 0, message });
      }
    }
  }

  return errors;
}

function main(argv) {
  const files = argv.filter((arg) => !arg.startsWith("-"));
  const errors = findParseErrors(files);

  if (errors.length === 0) return 0;

  for (const { file, line, column, message } of errors) {
    const location = line > 0 ? `${line}:${column}` : "";
    console.error(`${path.relative(process.cwd(), file)}:${location} ${message}`);
  }
  const fileCount = new Set(errors.map((e) => e.file)).size;
  console.error(
    `\n✖ ${errors.length} parse error(s) in ${fileCount} file(s). Fix them before committing.`
  );
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exit(main(process.argv.slice(2)));
}

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));

function parseArguments(argv) {
  const options = {
    root: path.resolve(scriptDirectory, ".."),
    contract: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--root") {
      const value = argv[index + 1];
      if (!value) throw new Error("--root 需要路径参数");
      options.root = path.resolve(value);
      index += 1;
      continue;
    }
    if (argument === "--contract") {
      const value = argv[index + 1];
      if (!value) throw new Error("--contract 需要路径参数");
      options.contract = path.resolve(value);
      index += 1;
      continue;
    }
    throw new Error(`未知参数：${argument}`);
  }

  if (!options.contract) {
    options.contract = path.join(options.root, "architecture", "command-contract.json");
  }
  return options;
}

function normalizeRelativePath(value) {
  return value.split(path.sep).join("/").replace(/^\.\//, "");
}

function resolveRepositoryFile(root, relativePath, label) {
  if (typeof relativePath !== "string" || relativePath.trim() === "") {
    throw new Error(`${label} 必须是非空路径`);
  }
  const normalized = normalizeRelativePath(relativePath);
  if (
    path.isAbsolute(relativePath) ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../")
  ) {
    throw new Error(`${label} 不能越出仓库根目录：${relativePath}`);
  }
  const absolute = path.join(root, normalized);
  if (!fs.existsSync(absolute)) {
    throw new Error(`${label} 不存在：${normalized}`);
  }
  if (!fs.lstatSync(absolute).isFile()) {
    throw new Error(`${label} 不是普通文件：${normalized}`);
  }
  return { absolute, relative: normalized };
}

function readText(root, relativePath, label) {
  const file = resolveRepositoryFile(root, relativePath, label);
  return { ...file, text: fs.readFileSync(file.absolute, "utf8") };
}

function readContract(contractPath) {
  let contract;
  try {
    contract = JSON.parse(fs.readFileSync(contractPath, "utf8"));
  } catch (error) {
    throw new Error(`无法读取命令契约 ${contractPath}：${error.message}`);
  }
  if (contract.contract_version !== "1.0.0") {
    throw new Error(`不支持的命令契约版本：${contract.contract_version ?? "缺失"}`);
  }
  if (!Array.isArray(contract.commands_in_registration_order)) {
    throw new Error("commands_in_registration_order 必须是数组");
  }
  if (!Array.isArray(contract.command_groups) || contract.command_groups.length === 0) {
    throw new Error("command_groups 必须是非空数组");
  }
  return contract;
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function duplicateValues(values) {
  const seen = new Set();
  const duplicates = new Set();
  for (const value of values) {
    if (seen.has(value)) duplicates.add(value);
    seen.add(value);
  }
  return [...duplicates].sort();
}

function compareSets(label, expectedValues, actualValues, failures) {
  const expected = new Set(expectedValues);
  const actual = new Set(actualValues);
  const missing = [...expected].filter((value) => !actual.has(value)).sort();
  const unexpected = [...actual].filter((value) => !expected.has(value)).sort();

  if (missing.length > 0) {
    failures.push(`${label} 缺少命令：${missing.join(", ")}`);
  }
  if (unexpected.length > 0) {
    failures.push(`${label} 存在孤立命令：${unexpected.join(", ")}`);
  }
}

function extractFrontendCommands(text, allowedDynamicArgument, failures) {
  const commands = [];
  const literalPattern =
    /\b(?:invoke|tauriInvoke)(?:\s*<[^;()]*?>)?\s*\(\s*["'`]([a-zA-Z0-9_]+)["'`]/g;
  for (const match of text.matchAll(literalPattern)) {
    commands.push(match[1]);
  }

  const transportPattern = /\btauriInvoke(?:\s*<[^;()]*?>)?\s*\(\s*([^,\n)]+)/g;
  for (const match of text.matchAll(transportPattern)) {
    const firstArgument = match[1].trim();
    if (/^["'`]/.test(firstArgument)) continue;
    if (firstArgument !== allowedDynamicArgument) {
      failures.push(`前端存在未授权的动态 Tauri 命令参数：${firstArgument}`);
    }
  }

  return commands;
}

function extractRegisteredCommands(text, failures) {
  const macroMatches = [...text.matchAll(/tauri::generate_handler!\s*\[([\s\S]*?)\]\s*\)/g)];
  if (macroMatches.length !== 1) {
    failures.push(`应且仅应存在一个 tauri::generate_handler!，实际 ${macroMatches.length}`);
    return [];
  }
  return [...macroMatches[0][1].matchAll(/\bcommands::([a-zA-Z0-9_]+)\b/g)].map(
    (match) => match[1],
  );
}

function extractRustCommandDefinitions(text) {
  const pattern =
    /#\s*\[\s*tauri::command(?:\s*\([^)]*\))?\s*\]\s*pub(?:\s*\([^)]*\))?\s+(?:async\s+)?fn\s+([a-zA-Z0-9_]+)/g;
  return [...text.matchAll(pattern)].map((match) => match[1]);
}

function verifyModuleRoot(text, groups, failures) {
  const declared = [...text.matchAll(/^\s*mod\s+([a-zA-Z0-9_]+)\s*;/gm)].map(
    (match) => match[1],
  );
  const exported = [...text.matchAll(/^\s*pub\s+use\s+([a-zA-Z0-9_]+)::\*\s*;/gm)].map(
    (match) => match[1],
  );
  const expected = groups.map((group) => group.module);

  const duplicateDeclarations = duplicateValues(declared);
  if (duplicateDeclarations.length > 0) {
    failures.push(`Rust 命令模块重复声明：${duplicateDeclarations.join(", ")}`);
  }
  const duplicateExports = duplicateValues(exported);
  if (duplicateExports.length > 0) {
    failures.push(`Rust 命令模块重复导出：${duplicateExports.join(", ")}`);
  }

  compareSets("Rust 命令模块声明", expected, declared, failures);
  compareSets("Rust 命令模块导出", expected, exported, failures);
}

function verify(options) {
  const contract = readContract(options.contract);
  const failures = [];

  const expectedCommands = contract.commands_in_registration_order;
  const duplicateContractCommands = duplicateValues(expectedCommands);
  if (duplicateContractCommands.length > 0) {
    failures.push(`命令契约存在重复命令：${duplicateContractCommands.join(", ")}`);
  }
  if (contract.command_count !== expectedCommands.length) {
    failures.push(
      `命令数量不一致：contract.command_count=${contract.command_count}，实际列表=${expectedCommands.length}`,
    );
  }

  const commandSetHash = sha256(`${expectedCommands.join("\n")}\n`);
  if (commandSetHash !== contract.command_set_sha256) {
    failures.push(
      `命令集合 SHA-256 变化：期望 ${contract.command_set_sha256}，实际 ${commandSetHash}`,
    );
  }

  const groupedCommands = [];
  const moduleNames = [];
  const rustDefinitions = [];
  const definitionOwners = new Map();

  for (const group of contract.command_groups) {
    if (!group || typeof group !== "object") {
      failures.push("command_groups 包含无效条目");
      continue;
    }
    if (typeof group.module !== "string" || group.module.length === 0) {
      failures.push("command_groups.module 必须是非空字符串");
      continue;
    }
    moduleNames.push(group.module);
    if (!Array.isArray(group.commands) || group.commands.length === 0) {
      failures.push(`命令组 ${group.module} 的 commands 必须是非空数组`);
      continue;
    }
    groupedCommands.push(...group.commands);

    const definition = readText(
      options.root,
      group.definition_file,
      `命令组 ${group.module} definition_file`,
    );
    resolveRepositoryFile(
      options.root,
      group.parameter_and_return_signature_file,
      `命令组 ${group.module} parameter_and_return_signature_file`,
    );
    resolveRepositoryFile(
      options.root,
      group.frontend_parameter_and_return_signature_file,
      `命令组 ${group.module} frontend_parameter_and_return_signature_file`,
    );
    resolveRepositoryFile(
      options.root,
      group.frontend_shared_dto_file,
      `命令组 ${group.module} frontend_shared_dto_file`,
    );

    const definitions = extractRustCommandDefinitions(definition.text);
    const duplicateDefinitions = duplicateValues(definitions);
    if (duplicateDefinitions.length > 0) {
      failures.push(
        `Rust 命令文件 ${definition.relative} 存在重复定义：${duplicateDefinitions.join(", ")}`,
      );
    }

    compareSets(
      `命令组 ${group.module}（${definition.relative}）`,
      group.commands,
      definitions,
      failures,
    );

    for (const command of definitions) {
      rustDefinitions.push(command);
      const owner = definitionOwners.get(command);
      if (owner) {
        failures.push(`Rust 命令 ${command} 同时定义于 ${owner} 和 ${definition.relative}`);
      } else {
        definitionOwners.set(command, definition.relative);
      }
    }
  }

  const definitionMapInput = contract.command_groups
    .flatMap((group) =>
      group.commands.map((command) => `${command}\0${normalizeRelativePath(group.definition_file)}\n`),
    )
    .sort()
    .join("");
  const definitionMapHash = sha256(definitionMapInput);
  if (definitionMapHash !== contract.command_definition_map_sha256) {
    failures.push(
      `命令定义映射 SHA-256 变化：期望 ${contract.command_definition_map_sha256}，实际 ${definitionMapHash}`,
    );
  }

  const duplicateModules = duplicateValues(moduleNames);
  if (duplicateModules.length > 0) {
    failures.push(`命令契约存在重复模块：${duplicateModules.join(", ")}`);
  }
  const duplicateGroupedCommands = duplicateValues(groupedCommands);
  if (duplicateGroupedCommands.length > 0) {
    failures.push(`命令分组重复归属：${duplicateGroupedCommands.join(", ")}`);
  }
  compareSets("命令分组", expectedCommands, groupedCommands, failures);

  const frontendSource = readText(
    options.root,
    contract.sources.frontend.api_file,
    "frontend.api_file",
  );
  resolveRepositoryFile(
    options.root,
    contract.sources.frontend.parameter_and_return_signature_file,
    "frontend.parameter_and_return_signature_file",
  );
  resolveRepositoryFile(
    options.root,
    contract.sources.frontend.shared_dto_file,
    "frontend.shared_dto_file",
  );
  const frontendCommands = extractFrontendCommands(
    frontendSource.text,
    contract.sources.frontend.allowed_dynamic_transport_argument,
    failures,
  );
  compareSets("前端命令调用集合", expectedCommands, frontendCommands, failures);

  const registrationSource = readText(
    options.root,
    contract.sources.tauri_registration.file,
    "tauri_registration.file",
  );
  const registeredCommands = extractRegisteredCommands(registrationSource.text, failures);
  const duplicateRegistrations = duplicateValues(registeredCommands);
  if (duplicateRegistrations.length > 0) {
    failures.push(`generate_handler! 重复注册：${duplicateRegistrations.join(", ")}`);
  }
  compareSets("Tauri 注册集合", expectedCommands, registeredCommands, failures);
  if (
    registeredCommands.length === expectedCommands.length &&
    registeredCommands.some((command, index) => command !== expectedCommands[index])
  ) {
    failures.push("generate_handler! 注册顺序与命令契约不一致");
  }

  const moduleRoot = readText(
    options.root,
    contract.sources.rust_module_root.file,
    "rust_module_root.file",
  );
  verifyModuleRoot(moduleRoot.text, contract.command_groups, failures);

  const duplicateRustDefinitions = duplicateValues(rustDefinitions);
  if (duplicateRustDefinitions.length > 0) {
    failures.push(`Rust 命令定义重复：${duplicateRustDefinitions.join(", ")}`);
  }
  compareSets("Rust #[tauri::command] 定义集合", expectedCommands, rustDefinitions, failures);

  if (failures.length > 0) {
    console.error("命令契约验证失败：");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exitCode = 1;
    return;
  }

  console.log(
    `命令契约验证通过：${expectedCommands.length} 个命令在前端调用、Rust 定义和 generate_handler! 注册三方完全一致；${contract.command_groups.length} 个命令模块无重复、缺失或孤立命令。`,
  );
}

try {
  verify(parseArguments(process.argv.slice(2)));
} catch (error) {
  console.error(`命令契约验证无法执行：${error.message}`);
  process.exitCode = 1;
}

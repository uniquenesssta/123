export class VerificationReport {
  constructor(name) {
    this.name = name;
    this.violations = [];
    this.notes = [];
  }

  check(condition, message) {
    if (!condition) this.violations.push(message);
  }

  violation(message) {
    this.violations.push(message);
  }

  note(message) {
    this.notes.push(message);
  }

  finish(summary) {
    for (const note of this.notes) console.log(`[architecture:note] ${note}`);
    if (this.violations.length > 0) {
      console.error(`\n[architecture] ${this.name} 失败，共 ${this.violations.length} 项：`);
      for (const violation of this.violations) console.error(`- ${violation}`);
      process.exit(1);
    }

    console.log(`[architecture] ${this.name} 通过：${summary}`);
  }
}

import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("EventCatalog package structure", () => {
	const packageRoot = resolve(__dirname, "..");

	it("has eventcatalog.config.js", () => {
		const configPath = resolve(packageRoot, "eventcatalog.config.js");
		expect(existsSync(configPath)).toBe(true);
	});

	it("has package.json with required scripts", async () => {
		const packageJsonPath = resolve(packageRoot, "package.json");
		expect(existsSync(packageJsonPath)).toBe(true);

		const packageJson = await import(packageJsonPath);
		expect(packageJson.scripts).toBeDefined();
		expect(packageJson.scripts.dev).toBe("eventcatalog dev");
		expect(packageJson.scripts.build).toBe("eventcatalog build");
	});

	it("has content directories", () => {
		// Core directories required for EventCatalog operation
		// Note: users/ is optional (individual contributor profiles vs team ownership)
		const requiredDirs = ["domains", "channels", "teams"];
		for (const dir of requiredDirs) {
			const dirPath = resolve(packageRoot, dir);
			expect(existsSync(dirPath), `${dir} directory should exist`).toBe(true);
		}
	});

	it("has real domain content", () => {
		// Verify transformed Qlerify content exists (not just scaffolding)
		const domainDirs = ["Session", "Analytics", "Workspace"];
		for (const domain of domainDirs) {
			const domainPath = resolve(packageRoot, "domains", domain, "index.mdx");
			expect(existsSync(domainPath), `${domain} domain should exist`).toBe(
				true,
			);
		}
	});
});

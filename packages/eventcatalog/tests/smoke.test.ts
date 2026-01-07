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
		const requiredDirs = ["domains", "channels", "teams", "users"];
		for (const dir of requiredDirs) {
			const dirPath = resolve(packageRoot, dir);
			expect(existsSync(dirPath), `${dir} directory should exist`).toBe(true);
		}
	});
});

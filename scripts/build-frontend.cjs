const { cpSync, mkdirSync } = require("node:fs");
const path = require("node:path");
const root = path.resolve(__dirname, "..");
const dist = path.join(root, "dist");
mkdirSync(dist, { recursive: true });
for (const file of ["index.html", "styles.css", "renderer.js", "api.js", "format.js", "appearance.js", "confirmation.js", "window.js"]) {
  cpSync(path.join(root, "src", file), path.join(dist, file));
}
cpSync(path.join(root, "assets"), path.join(dist, "assets"), { recursive: true });

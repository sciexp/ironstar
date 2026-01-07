/** @type {import('@eventcatalog/core/bin/eventcatalog.config').Config} */
export default {
	title: "Ironstar Event Catalog",
	tagline:
		"Event-driven architecture documentation for the Ironstar scientific data analysis platform. Discover domains, explore services and their dependencies, and understand the message contracts that connect our infrastructure.",
	organizationName: "Ironstar",
	homepageLink: "https://github.com/sciexp/ironstar",
	editUrl: "https://github.com/sciexp/ironstar/edit/main/packages/eventcatalog",
	// Static rendering for Cloudflare Pages deployment
	output: "static",
	trailingSlash: false,
	// Base path - adjust if deploying to subdirectory
	base: "/",
	logo: {
		alt: "Ironstar",
		src: "/logo.png",
		text: "Ironstar",
	},
	rss: {
		enabled: true,
		limit: 20,
	},
	// Enable LLM-friendly text export for AI assistants
	llmsTxt: {
		enabled: true,
	},
	docs: {
		sidebar: {
			// TREE_VIEW maps file system structure
			// LIST_VIEW provides API documentation style
			type: "TREE_VIEW",
		},
	},
	// Required unique identifier for EventCatalog
	cId: "7237ddc2-5ceb-47b6-a819-0c15aae89d4f",
};

import type { NextConfig } from "next";

const nextConfig: NextConfig = {
	typedRoutes: true,
	reactCompiler: true,
	reactStrictMode: false,
	images: {
		qualities: [75, 95],
	},
};

export default nextConfig;

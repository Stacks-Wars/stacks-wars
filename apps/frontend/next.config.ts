import type { NextConfig } from "next";

const nextConfig: NextConfig = {
	images: {
		remotePatterns: [{ protocol: "https", hostname: "res.cloudinary.com" }],
		formats: ["image/avif", "image/webp"],
	},
	serverExternalPackages: ["grammy"],
	/* config options here */
};

export default nextConfig;

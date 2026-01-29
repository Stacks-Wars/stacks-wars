import type { NextConfig } from "next";

const nextConfig: NextConfig = {
typedRoutes: true,
// Temporarily disabled due to Turbopack crash
// reactCompiler: true,
turbopack: {
root: "/home/ipeter/stacks-wars",
},
async rewrites() {
return [
{
source: "/api/:path*",
destination: "http://localhost:3001/api/:path*",
},
];
},
};

export default nextConfig;

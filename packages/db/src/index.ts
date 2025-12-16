import * as schema from "./schema";

const isProd = process.env.NODE_ENV === "production";

async function neonDb() {
	const { neon, neonConfig } = await import("@neondatabase/serverless");
	const { drizzle } = await import("drizzle-orm/neon-http");
	const ws = await import("ws");

	neonConfig.webSocketConstructor = ws;
	neonConfig.poolQueryViaFetch = true;

	const sql = neon(process.env.DATABASE_URL || "");
	return drizzle(sql, { schema });
}

async function nodePostgresDb() {
	const { drizzle } = await import("drizzle-orm/node-postgres");

	return drizzle(process.env.DATABASE_URL || "", { schema });
}

export const db = isProd ? await neonDb() : await nodePostgresDb();

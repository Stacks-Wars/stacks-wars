import { expo } from "@better-auth/expo";
import { db } from "@stacks-wars/db";
import * as schema from "@stacks-wars/db/schema/auth";
import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
import { nextCookies } from "better-auth/next-js";

export const auth = betterAuth({
	database: drizzleAdapter(db, {
		provider: "pg",

		schema: schema,
	}),
	trustedOrigins: [process.env.CORS_ORIGIN || "", "exp://"],
	emailAndPassword: {
		enabled: true,
	},
	plugins: [nextCookies(), expo()],
});

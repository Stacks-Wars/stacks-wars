import { expo } from "@better-auth/expo";
import { verifyMessageSignatureRsv } from "@stacks/encryption";
import { db } from "@stacks-wars/db";
import * as schema from "@stacks-wars/db/schema/auth";
import { BETTERAUTH_URL, DOMAIN_NAME } from "@stacks-wars/shared";
import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
import { generateRandomString } from "better-auth/crypto";
import { nextCookies } from "better-auth/next-js";
import { bearer, jwt, openAPI } from "better-auth/plugins";
import { siws } from "./plugins/siws";
import { getBnsAndAvatar } from "./plugins/siws/utils";

export const auth = betterAuth({
	database: drizzleAdapter(db, {
		provider: "pg",
		schema: schema,
	}),
	trustedOrigins: [BETTERAUTH_URL || "", "exp://"],
	emailAndPassword: {
		enabled: true,
		requireEmailVerification: true,
		autoSignIn: true,
		minPasswordLength: 8,
		revokeSessionsOnPasswordReset: true,
		resetPasswordTokenExpiresIn: 10 * 60,
	},
	plugins: [
		bearer(),
		siws({
			domain: DOMAIN_NAME,
			emailDomainName: DOMAIN_NAME,
			getNonce: async () => {
				return generateRandomString(32);
			},
			bnsLookup: async ({ walletAddress }) => {
				try {
					const res = await getBnsAndAvatar(walletAddress);

					if (!res || typeof res !== "object") {
						throw new Error("Invalid BNS response");
					}

					return {
						name: res.name || walletAddress,
						avatar: res.avatar || "",
					};
				} catch (err) {
					console.error(`BNS lookup failed for ${walletAddress}:`, err);

					return {
						name: walletAddress,
						avatar: "",
					};
				}
			},
			verifyMessage: async ({
				message,
				signature,
				publicKey,
				address,
				nonce,
			}) => {
				try {
					const isValidSignature = verifyMessageSignatureRsv({
						message,
						signature,
						publicKey,
					});

					if (!isValidSignature) {
						console.error("Signature verification failed", {
							address: address?.substring(0, 10) + "...",
							signaturePrefix: signature?.substring(0, 20) + "...",
						});
						return false;
					}

					return true;
				} catch (error) {
					console.error("SIWS verification error:", {
						error: error instanceof Error ? error.message : "Unknown error",
						address: address?.substring(0, 10) + "...",
					});
					return false;
				}
			},
		}),
		jwt({
			jwt: {
				expirationTime: "15m",
				audience: BETTERAUTH_URL,
				issuer: BETTERAUTH_URL,
			},
		}),
		openAPI(),
		expo(),
		nextCookies(),
	],
});

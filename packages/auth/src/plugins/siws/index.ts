//plugin/index.ts
import { validateStacksAddress } from "@stacks/transactions";
import type { BetterAuthPlugin, User } from "better-auth";
import { APIError } from "better-auth/api";
import { setSessionCookie } from "better-auth/cookies";
import { createAuthEndpoint } from "better-auth/plugins";
import { z } from "zod";
import { schema } from "./schema";
import type { SIWSPluginOptions, WalletAddress } from "./types";

export const siws = (options: SIWSPluginOptions) =>
	({
		id: "siws",
		schema: schema,
		endpoints: {
			getSiwsNonce: createAuthEndpoint(
				"/siws/nonce",
				{
					method: "POST",
					body: z.object({
						walletAddress: z
							.string()
							.refine((address) => validateStacksAddress(address), {
								message: "Invalid Stacks wallet address format",
							}),
					}),
				},
				async (ctx) => {
					const { walletAddress } = ctx.body;
					const nonce = await options.getNonce();

					await ctx.context.internalAdapter.createVerificationValue({
						identifier: `siws:${walletAddress}`,
						value: nonce,
						expiresAt: new Date(Date.now() + 15 * 60 * 1000),
					});

					return ctx.json({ nonce });
				},
			),

			verifySiwsMessage: createAuthEndpoint(
				"/siws/verify",
				{
					method: "POST",
					body: z
						.object({
							message: z.string().min(1),
							signature: z.string().min(1),
							walletAddress: z
								.string()
								.refine((address) => validateStacksAddress(address), {
									message: "Invalid Stacks wallet address format",
								}),
							publicKey: z.string(),
							email: z.email().optional(),
						})
						.refine((data) => options.anonymous !== false || !!data.email, {
							message: "Email is required when anonymous is disabled.",
							path: ["email"],
						}),
					requireRequest: true,
				},
				async (ctx) => {
					const { message, signature, walletAddress, email, publicKey } =
						ctx.body;
					const isAnon = options.anonymous ?? true;

					if (!isAnon && !email) {
						throw ctx.error("BAD_REQUEST", {
							message: "Email is required when anonymous is disabled.",
							status: 400,
						});
					}

					try {
						const verification =
							await ctx.context.internalAdapter.findVerificationValue(
								`siws:${walletAddress}`,
							);

						if (!verification || new Date() > verification.expiresAt) {
							throw ctx.error("UNAUTHORIZED", {
								message: "Invalid or expired nonce",
								status: 401,
								code: "UNAUTHORIZED_INVALID_OR_EXPIRED_NONCE",
							});
						}

						const verified = await options.verifyMessage({
							message,
							signature,
							address: walletAddress,
							nonce: verification.value,
							publicKey: publicKey,
						});

						if (!verified) {
							throw ctx.error("UNAUTHORIZED", {
								message: "Unauthorized: Invalid SIWS signature",
								status: 401,
							});
						}

						await ctx.context.internalAdapter.deleteVerificationValue(
							verification.id,
						);

						let user: User | null = null;
						const existingWallet: WalletAddress | null =
							await ctx.context.adapter.findOne({
								model: "walletAddress",
								where: [
									{ field: "address", operator: "eq", value: walletAddress },
								],
							});

						const network = walletAddress.startsWith("SP")
							? "mainnet"
							: "testnet";

						if (existingWallet) {
							user = await ctx.context.adapter.findOne({
								model: "user",
								where: [
									{ field: "id", operator: "eq", value: existingWallet.userId },
								],
							});
						}

						if (!user) {
							const domain = options.emailDomainName;
							const userEmail =
								!isAnon && email ? email : `${walletAddress}@${domain}`;
							const { name, avatar } =
								(await options.bnsLookup?.({ walletAddress })) ?? {};

							user = await ctx.context.internalAdapter.createUser({
								name: name ?? walletAddress,
								email: userEmail,
								image: avatar ?? "",
							});

							if (user) {
								// Any action that needs to run when user has been created can go here
							}

							await ctx.context.adapter.create({
								model: "walletAddress",
								data: {
									userId: user.id,
									address: walletAddress,
									network, // Use inferred network
									isPrimary: true,
									createdAt: new Date(),
								},
							});

							await ctx.context.internalAdapter.createAccount({
								userId: user.id,
								providerId: "siws",
								accountId: walletAddress,
								createdAt: new Date(),
								updatedAt: new Date(),
							});
						}

						const session = await ctx.context.internalAdapter.createSession(
							user.id,
						);
						if (!session) {
							throw ctx.error("INTERNAL_SERVER_ERROR", {
								message: "Internal Server Error",
								status: 500,
							});
						}

						await setSessionCookie(ctx, { session, user });

						return ctx.json({
							token: session.token,
							success: true,
							user: {
								id: user.id,
								walletAddress: walletAddress,
								network, // Return network instead of chainId
							},
						});
					} catch (err) {
						if (err instanceof APIError) throw err;
						throw ctx.error("UNAUTHORIZED", {
							message: "Something went wrong.",
							status: 401,
							error: err instanceof Error ? err.message : "Unknown error",
						});
					}
				},
			),
		},
	}) satisfies BetterAuthPlugin;

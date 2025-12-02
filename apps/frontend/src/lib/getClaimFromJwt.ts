"use server";

import { disconnect, isConnected } from "@stacks/connect";
import { cookies } from "next/headers";

interface JwtPayload {
	sub: string;
	wallet: string;
	exp: number;
}

type ClaimKey = "sub" | "wallet" | "exp";

export const getClaimFromJwt = async <T = number | string>(
	claim: ClaimKey
): Promise<T | null> => {
	try {
		const cookieStore = await cookies();
		const jwt = cookieStore.get("jwt")?.value;
		if (!jwt) {
			if (isConnected()) {
				disconnect();
			}
			return null;
		}

		const payload = jwt.split(".")[1];
		if (!payload) return null;

		const decoded = atob(payload);
		const data = JSON.parse(decoded) as JwtPayload;

		return (data[claim] ?? null) as T | null;
	} catch (err) {
		console.error("Failed to parse JWT:", err);
		return null;
	}
};

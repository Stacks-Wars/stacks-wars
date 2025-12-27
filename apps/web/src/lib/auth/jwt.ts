import { jwtVerify } from "jose";
import { cookies } from "next/headers";

const JWT_SECRET = process.env.JWT_SECRET;

interface JWTPayload {
	sub: string; // user id
	iat: number;
	exp: number;
	jti: string;
}

export async function verifyToken(token: string): Promise<JWTPayload> {
	const secret = new TextEncoder().encode(JWT_SECRET);

	const { payload } = await jwtVerify(token, secret);

	return {
		sub: payload.sub as string,
		iat: payload.iat as number,
		exp: payload.exp as number,
		jti: payload.jti as string,
	};
}

/**
 * Gets the authenticated user's ID from the JWT token cookie.
 * Returns the user ID if authenticated, null otherwise.
 */
export async function getAuthenticatedUserId(): Promise<string | null> {
	try {
		const cookieStore = await cookies();
		const token = cookieStore.get("auth_token")?.value;

		if (!token) {
			return null;
		}

		const decoded = await verifyToken(token);
		return decoded.sub;
	} catch (error) {
		// Token invalid or expired
		return null;
	}
}

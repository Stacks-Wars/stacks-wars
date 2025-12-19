"use client";

const TOKEN_COOKIE_NAME = "auth_token";
const MAX_AGE = 60 * 60 * 24 * 7; // 7 days

export function setAuthToken(token: string) {
	// Set as httpOnly=false for client access, but with secure flags
	document.cookie = `${TOKEN_COOKIE_NAME}=${token}; path=/; max-age=${MAX_AGE}; SameSite=Strict${
		process.env.NODE_ENV === "production" ? "; Secure" : ""
	}`;
}

export function getAuthToken(): string | null {
	// Return null on server side
	if (typeof window === "undefined" || typeof document === "undefined")
		return null;

	const cookies = document.cookie.split("; ");
	const tokenCookie = cookies.find((cookie) =>
		cookie.startsWith(`${TOKEN_COOKIE_NAME}=`)
	);

	if (!tokenCookie) return null;

	return tokenCookie.split("=")[1];
}

export function removeAuthToken() {
	document.cookie = `${TOKEN_COOKIE_NAME}=; path=/; max-age=0`;
}

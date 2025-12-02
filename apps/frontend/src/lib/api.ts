"use server";

import { revalidatePath, revalidateTag } from "next/cache";
import { cookies } from "next/headers";

type JSONValue = string | number | boolean | null | JSONObject | JSONValue[];

interface JSONObject {
	[key: string]: JSONValue;
}

type JSON = JSONValue;

export interface ApiRequestProps {
	path: string;
	method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
	body?: JSONObject;
	tag?: string | string[];
	revalidateTag?: string;
	revalidatePath?: string;
	cache?: "no-store" | "force-cache";
	auth?: boolean;
}

export const apiRequest = async <T = JSON>({
	path,
	method = "GET",
	body,
	tag,
	revalidateTag: tagToRevalidate,
	revalidatePath: pathToRevalidate,
	cache,
	auth = true,
}: ApiRequestProps): Promise<T> => {
	let headers: HeadersInit;

	if (auth) {
		const cookieStore = await cookies();
		const jwt = cookieStore.get("jwt");

		if (!jwt) {
			throw new Error("You must be logged in to perform this action");
		}

		headers = {
			"Content-Type": "application/json",
			Authorization: `Bearer ${jwt.value}`,
		};
	} else {
		headers = {
			"Content-Type": "application/json",
		};
	}

	const normalizedPath = path.startsWith("/") ? path : `/${path}`;

	const res = await fetch(
		`${process.env.NEXT_PUBLIC_API_URL}${normalizedPath}`,
		{
			method,
			headers,
			body: body ? JSON.stringify(body) : undefined,
			...(cache && { cache }),
			next: tag ? { tags: Array.isArray(tag) ? tag : [tag] } : undefined,
		}
	);

	if (!res.ok) {
		const text = await res.text();
		if (res.status === 401)
			throw new Error("Unauthorized – maybe you need to log in again?");
		if (res.status === 403)
			throw new Error("Forbidden – you don’t have access.");
		throw new Error(text || "Request failed");
	}

	if (tagToRevalidate) revalidateTag(tagToRevalidate);
	if (pathToRevalidate) revalidatePath(pathToRevalidate);

	return res.json() as Promise<T>;
};
